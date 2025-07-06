#![allow(dead_code)]

use colored::*;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Simple progress bar for CLI operations
pub struct ProgressBar {
    message: String,
    start_time: Instant,
    spinner_chars: Vec<char>,
    current_char: usize,
    is_running: Arc<AtomicBool>,
}

impl ProgressBar {
    pub fn new(message: String) -> Self {
        Self {
            message,
            start_time: Instant::now(),
            spinner_chars: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current_char: 0,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the progress bar in a background task
    pub async fn start(&mut self) -> ProgressHandle {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let message = self.message.clone();
        let spinner_chars = self.spinner_chars.clone();

        let handle = tokio::spawn(async move {
            let mut char_index = 0;
            let start_time = Instant::now();

            while is_running.load(Ordering::SeqCst) {
                let elapsed = start_time.elapsed();
                let spinner_char = spinner_chars[char_index % spinner_chars.len()];

                print!(
                    "\r{} {message} ({})",
                    spinner_char.to_string().cyan(),
                    format_duration(elapsed).dimmed()
                );
                io::stdout().flush().unwrap_or(());

                char_index += 1;
                sleep(Duration::from_millis(100)).await;
            }
        });

        ProgressHandle {
            task_handle: handle,
            is_running: Arc::clone(&self.is_running),
            start_time: self.start_time,
        }
    }
}

/// Handle to control a running progress bar
pub struct ProgressHandle {
    task_handle: tokio::task::JoinHandle<()>,
    is_running: Arc<AtomicBool>,
    start_time: Instant,
}

impl ProgressHandle {
    /// Stop the progress bar with a success message
    pub async fn finish_with_message(self, message: &str) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.task_handle.await;

        let elapsed = self.start_time.elapsed();
        println!(
            "\r{} {message} ({})",
            "✓".green(),
            format_duration(elapsed).dimmed()
        );
    }

    /// Stop the progress bar with an error message
    pub async fn finish_with_error(self, message: &str) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.task_handle.await;

        let elapsed = self.start_time.elapsed();
        println!(
            "\r{} {message} ({})",
            "✗".red(),
            format_duration(elapsed).dimmed()
        );
    }

    /// Stop the progress bar with a warning message
    pub async fn finish_with_warning(self, message: &str) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.task_handle.await;

        let elapsed = self.start_time.elapsed();
        println!(
            "\r{} {message} ({})",
            "⚠".yellow(),
            format_duration(elapsed).dimmed()
        );
    }
}

/// Multi-step progress tracker for complex operations
pub struct MultiStepProgress {
    steps: Vec<ProgressStep>,
    current_step: usize,
    total_steps: usize,
    start_time: Instant,
}

#[derive(Clone)]
struct ProgressStep {
    name: String,
    status: StepStatus,
    duration: Option<Duration>,
}

#[derive(Clone, PartialEq)]
enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl MultiStepProgress {
    pub fn new(step_names: Vec<String>) -> Self {
        let total_steps = step_names.len();
        let steps = step_names
            .into_iter()
            .map(|name| ProgressStep {
                name,
                status: StepStatus::Pending,
                duration: None,
            })
            .collect();

        Self {
            steps,
            current_step: 0,
            total_steps,
            start_time: Instant::now(),
        }
    }

    /// Start the next step
    pub fn start_step(&mut self, _step_name: &str) -> Option<StepHandle> {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].status = StepStatus::InProgress;
            self.print_progress();

            Some(StepHandle {
                step_index: self.current_step,
                start_time: Instant::now(),
            })
        } else {
            None
        }
    }

    /// Complete the current step
    pub fn complete_step(&mut self, handle: StepHandle) {
        if handle.step_index < self.steps.len() {
            self.steps[handle.step_index].status = StepStatus::Completed;
            self.steps[handle.step_index].duration = Some(handle.start_time.elapsed());
            self.current_step += 1;
            self.print_progress();
        }
    }

    /// Fail the current step
    pub fn fail_step(&mut self, handle: StepHandle, error_message: &str) {
        if handle.step_index < self.steps.len() {
            self.steps[handle.step_index].status = StepStatus::Failed;
            self.steps[handle.step_index].duration = Some(handle.start_time.elapsed());
            self.print_progress();
            println!("   {} {error_message}", "Error:".red().bold());
        }
    }

    /// Skip the current step
    pub fn skip_step(&mut self, handle: StepHandle, reason: &str) {
        if handle.step_index < self.steps.len() {
            self.steps[handle.step_index].status = StepStatus::Skipped;
            self.steps[handle.step_index].duration = Some(handle.start_time.elapsed());
            self.current_step += 1;
            self.print_progress();
            println!("   {} {reason}", "Skipped:".yellow().bold());
        }
    }

    /// Print the current progress state
    fn print_progress(&self) {
        println!("\n{}", "📋 Progress Overview".bold());
        println!("{}", "─".repeat(50));

        for (i, step) in self.steps.iter().enumerate() {
            let (symbol, color_fn): (&str, fn(&str) -> ColoredString) = match step.status {
                StepStatus::Pending => ("○", |s| s.dimmed()),
                StepStatus::InProgress => ("⟳", |s| s.cyan()),
                StepStatus::Completed => ("✓", |s| s.green()),
                StepStatus::Failed => ("✗", |s| s.red()),
                StepStatus::Skipped => ("⊘", |s| s.yellow()),
            };

            let duration_str = step
                .duration
                .map(|d| format!(" ({})", format_duration(d)))
                .unwrap_or_default();

            println!(
                "{} {}/{} {} {}{}",
                color_fn(symbol),
                i + 1,
                self.total_steps,
                color_fn(&step.name),
                if step.status == StepStatus::InProgress {
                    "...".dimmed().to_string()
                } else {
                    String::new()
                },
                duration_str.dimmed()
            );
        }

        if self.current_step >= self.total_steps {
            let total_duration = self.start_time.elapsed();
            println!(
                "{}\n{} All steps completed in {}",
                "─".repeat(50),
                "🎉".green(),
                format_duration(total_duration).bold()
            );
        }
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        let completed = self
            .steps
            .iter()
            .filter(|s| matches!(s.status, StepStatus::Completed | StepStatus::Skipped))
            .count();

        if self.total_steps == 0 {
            100.0
        } else {
            (completed as f64 / self.total_steps as f64) * 100.0
        }
    }
}

pub struct StepHandle {
    step_index: usize,
    start_time: Instant,
}

/// Format duration in a human-readable way
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if total_secs >= 60 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{mins}m{secs}s")
    } else if total_secs > 0 {
        format!("{total_secs}.{}s", millis / 100)
    } else {
        format!("{millis}ms")
    }
}

/// Enhanced status reporting for CLI operations
pub struct StatusReporter {
    current_operation: Arc<RwLock<Option<String>>>,
}

impl StatusReporter {
    pub fn new() -> Self {
        Self {
            current_operation: Arc::new(RwLock::new(None)),
        }
    }

    /// Report successful operation
    pub async fn success(&self, message: &str) {
        println!("{} {message}", "✓".green().bold());
    }

    /// Report error
    pub async fn error(&self, message: &str) {
        println!("{} {message}", "✗".red().bold());
    }

    /// Report warning
    pub async fn warning(&self, message: &str) {
        println!("{} {message}", "⚠".yellow().bold());
    }

    /// Report info
    pub async fn info(&self, message: &str) {
        println!("{} {message}", "ℹ".blue().bold());
    }

    /// Show a tip or suggestion
    pub async fn tip(&self, message: &str) {
        println!("{} Tip: {message}", "💡".bright_yellow().bold());
    }

    /// Clear the current line (useful for progress updates)
    pub fn clear_line() {
        print!("\r{}\r", " ".repeat(80));
        io::stdout().flush().unwrap_or(());
    }
}

impl Default for StatusReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(1)), "1.0s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m5s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "61m1s");
    }

    #[test]
    fn test_multi_step_progress_creation() {
        let steps = vec![
            "Step 1".to_string(),
            "Step 2".to_string(),
            "Step 3".to_string(),
        ];
        let progress = MultiStepProgress::new(steps);

        assert_eq!(progress.total_steps, 3);
        assert_eq!(progress.current_step, 0);
        assert_eq!(progress.completion_percentage(), 0.0);
    }

    #[test]
    fn test_completion_percentage() {
        let steps = vec!["Step 1".to_string(), "Step 2".to_string()];
        let mut progress = MultiStepProgress::new(steps);

        // Complete first step
        if let Some(handle) = progress.start_step("Step 1") {
            progress.complete_step(handle);
        }

        assert_eq!(progress.completion_percentage(), 50.0);
    }

    #[tokio::test]
    async fn test_progress_bar() {
        let mut progress = ProgressBar::new("Testing...".to_string());
        let handle = progress.start().await;

        // Let it run briefly
        sleep(Duration::from_millis(50)).await;

        handle.finish_with_message("Test completed").await;
    }

    #[tokio::test]
    async fn test_status_reporter() {
        let reporter = StatusReporter::new();

        reporter.info("Testing status reporter").await;
        reporter.success("Operation successful").await;
        reporter.warning("This is a warning").await;
        reporter.error("This is an error").await;
        reporter.tip("This is a helpful tip").await;
    }
}
