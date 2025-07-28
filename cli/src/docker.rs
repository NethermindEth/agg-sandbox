use crate::error::{DockerError, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Detect which Docker Compose command is available
/// Returns "docker" if `docker compose` is available, otherwise "docker-compose"
fn get_compose_command() -> &'static str {
    // Test if `docker compose` is available (modern integrated version)
    if let Ok(output) = Command::new("docker")
        .args(["compose", "--version"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if the output actually contains compose-related information
            if stdout.to_lowercase().contains("compose") {
                return "docker";
            }
        }
    }

    // Fall back to standalone docker-compose
    if Command::new("docker-compose")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return "docker-compose";
    }

    // Default to docker-compose for backward compatibility
    "docker-compose"
}

/// Get the appropriate compose command arguments
/// Returns ("docker", ["compose"]) for modern Docker or ("docker-compose", []) for legacy
fn get_compose_command_parts() -> (&'static str, Vec<&'static str>) {
    if get_compose_command() == "docker" {
        ("docker", vec!["compose"])
    } else {
        ("docker-compose", vec![])
    }
}

/// Get a descriptive name for the compose command for error reporting
fn get_compose_command_name() -> String {
    if get_compose_command() == "docker" {
        "docker compose".to_string()
    } else {
        "docker-compose".to_string()
    }
}

/// Builder for Docker Compose commands with environment and file management
#[derive(Debug, Clone)]
pub struct DockerComposeBuilder {
    files: Vec<String>,
    env_vars: HashMap<String, String>,
    services: Vec<String>,
}

impl DockerComposeBuilder {
    /// Create a new Docker Compose builder
    pub fn new() -> Self {
        Self {
            files: vec!["docker-compose.yml".to_string()],
            env_vars: HashMap::new(),
            services: Vec::new(),
        }
    }

    /// Add a compose file to the command
    pub fn add_file<S: Into<String>>(&mut self, file: S) -> &mut Self {
        self.files.push(file.into());
        self
    }

    /// Set the compose files (replacing existing ones)
    pub fn set_files<S: Into<String>>(&mut self, files: Vec<S>) -> &mut Self {
        self.files = files.into_iter().map(|f| f.into()).collect();
        self
    }

    /// Add an environment variable
    pub fn add_env<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add a service to target (for logs, etc.)
    pub fn add_service<S: Into<String>>(&mut self, service: S) -> &mut Self {
        self.services.push(service.into());
        self
    }

    /// Build a docker-compose up command
    pub fn build_up_command(&self, detach: bool, build: bool) -> Command {
        let (program, base_args) = get_compose_command_parts();
        let mut cmd = Command::new(program);

        // Add base arguments (e.g., "compose" for modern docker command)
        for arg in base_args {
            cmd.arg(arg);
        }

        // Add compose files
        for file in &self.files {
            cmd.arg("-f").arg(file);
        }

        cmd.arg("up");

        if detach {
            cmd.arg("-d");
        }

        if build {
            cmd.arg("--build");
        }

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        cmd
    }

    /// Build a docker-compose down command
    pub fn build_down_command(&self, volumes: bool) -> Command {
        let (program, base_args) = get_compose_command_parts();
        let mut cmd = Command::new(program);

        // Add base arguments (e.g., "compose" for modern docker command)
        for arg in base_args {
            cmd.arg(arg);
        }

        // Add compose files
        for file in &self.files {
            cmd.arg("-f").arg(file);
        }

        cmd.arg("down");

        if volumes {
            cmd.arg("-v");
        }

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        cmd
    }

    /// Build a docker-compose ps command
    pub fn build_ps_command(&self) -> Command {
        let (program, base_args) = get_compose_command_parts();
        let mut cmd = Command::new(program);

        // Add base arguments (e.g., "compose" for modern docker command)
        for arg in base_args {
            cmd.arg(arg);
        }

        // Add compose files
        for file in &self.files {
            cmd.arg("-f").arg(file);
        }

        cmd.arg("ps");

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        cmd
    }

    /// Build a docker-compose logs command
    pub fn build_logs_command(&self, follow: bool) -> Command {
        let (program, base_args) = get_compose_command_parts();
        let mut cmd = Command::new(program);

        // Add base arguments (e.g., "compose" for modern docker command)
        for arg in base_args {
            cmd.arg(arg);
        }

        // Add compose files
        for file in &self.files {
            cmd.arg("-f").arg(file);
        }

        cmd.arg("logs");

        if follow {
            cmd.arg("-f");
        }

        // Add services if specified
        for service in &self.services {
            cmd.arg(service);
        }

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        cmd
    }
}

impl Default for DockerComposeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for sandbox modes
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub fork_mode: bool,
    pub multi_l2_mode: bool,
}

impl SandboxConfig {
    /// Create a new sandbox configuration
    pub fn new(fork_mode: bool, multi_l2_mode: bool) -> Self {
        Self {
            fork_mode,
            multi_l2_mode,
        }
    }

    /// Get the mode description for display
    pub fn mode_description(&self) -> &'static str {
        match (self.fork_mode, self.multi_l2_mode) {
            (true, true) => "multi-L2 fork mode",
            (true, false) => "fork mode",
            (false, true) => "multi-L2 mode",
            (false, false) => "local mode",
        }
    }

    /// Validate fork mode configuration
    pub fn validate_fork_config(&self) -> Result<()> {
        if !self.fork_mode {
            return Ok(());
        }

        // Check required fork URLs
        let fork_mainnet = std::env::var("FORK_URL_MAINNET").unwrap_or_default();
        let fork_agglayer_1 = std::env::var("FORK_URL_AGGLAYER_1").unwrap_or_default();

        if fork_mainnet.is_empty() {
            return Err(DockerError::compose_validation_failed(
                "FORK_URL_MAINNET environment variable is not set",
            )
            .into());
        }

        if fork_agglayer_1.is_empty() {
            return Err(DockerError::compose_validation_failed(
                "FORK_URL_AGGLAYER_1 environment variable is not set",
            )
            .into());
        }

        // Additional validation for multi-L2 fork mode
        if self.multi_l2_mode {
            let fork_agglayer_2 = std::env::var("FORK_URL_AGGLAYER_2").unwrap_or_default();
            if fork_agglayer_2.is_empty() {
                return Err(DockerError::compose_validation_failed(
                    "FORK_URL_AGGLAYER_2 environment variable is not set for multi-L2 fork mode",
                )
                .into());
            }
        }

        Ok(())
    }

    /// Create a DockerComposeBuilder configured for this sandbox mode
    pub fn create_docker_builder(&self) -> DockerComposeBuilder {
        let mut builder = DockerComposeBuilder::new();

        // In multi-L2 mode, use only the multi-L2 compose file
        if self.multi_l2_mode {
            builder.set_files(vec!["docker-compose.multi-l2.yml"]);
        }

        // Set environment variables based on mode
        if self.fork_mode {
            builder.add_env("ENABLE_FORK_MODE", "true");

            // Add fork URLs
            if let Ok(fork_mainnet) = std::env::var("FORK_URL_MAINNET") {
                builder.add_env("FORK_URL_MAINNET", fork_mainnet);
            }
            if let Ok(fork_agglayer_1) = std::env::var("FORK_URL_AGGLAYER_1") {
                builder.add_env("FORK_URL_AGGLAYER_1", fork_agglayer_1);
            }
            if self.multi_l2_mode {
                if let Ok(fork_agglayer_2) = std::env::var("FORK_URL_AGGLAYER_2") {
                    builder.add_env("FORK_URL_AGGLAYER_2", fork_agglayer_2);
                }
            }
        } else {
            builder.add_env("ENABLE_FORK_MODE", "false");
        }

        // Set chain IDs
        let chain_id_mainnet =
            std::env::var("CHAIN_ID_MAINNET").unwrap_or_else(|_| "1".to_string());
        let chain_id_agglayer_1 =
            std::env::var("CHAIN_ID_AGGLAYER_1").unwrap_or_else(|_| "1101".to_string());

        builder.add_env("CHAIN_ID_MAINNET", chain_id_mainnet);
        builder.add_env("CHAIN_ID_AGGLAYER_1", chain_id_agglayer_1);

        if self.multi_l2_mode {
            let chain_id_agglayer_2 =
                std::env::var("CHAIN_ID_AGGLAYER_2").unwrap_or_else(|_| "1102".to_string());
            builder.add_env("CHAIN_ID_AGGLAYER_2", chain_id_agglayer_2);
        }

        builder
    }
}

/// Create a DockerComposeBuilder that automatically detects multi-L2 configuration
pub fn create_auto_docker_builder() -> DockerComposeBuilder {
    let mut builder = DockerComposeBuilder::new();

    // Check if multi-L2 compose file exists and add it
    if Path::new("docker-compose.multi-l2.yml").exists() {
        builder.add_file("docker-compose.multi-l2.yml");
    }

    builder
}

/// Execute a Docker Compose command and handle output appropriately
pub fn execute_docker_command(mut command: Command, capture_output: bool) -> Result<()> {
    let cmd_name = get_compose_command_name();

    if capture_output {
        // Capture output for detached operations
        let output = command
            .output()
            .map_err(|e| DockerError::command_failed(&cmd_name, &e.to_string()))?;

        if !output.status.success() {
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !stderr_output.is_empty() {
                eprint!("{stderr_output}");
            }
            return Err(DockerError::command_failed(&cmd_name, &stderr_output).into());
        }
    } else {
        // Run in foreground with real-time output
        let status = command
            .status()
            .map_err(|e| DockerError::command_failed(&cmd_name, &e.to_string()))?;

        if !status.success() {
            return Err(DockerError::command_failed(
                &cmd_name,
                "Command exited with non-zero status",
            )
            .into());
        }
    }

    Ok(())
}

/// Execute a Docker Compose command and return output
pub fn execute_docker_command_with_output(mut command: Command) -> Result<String> {
    let cmd_name = get_compose_command_name();

    let output = command
        .output()
        .map_err(|e| DockerError::command_failed(&cmd_name, &e.to_string()))?;

    if !output.status.success() {
        let stderr_output = String::from_utf8_lossy(&output.stderr);
        if !stderr_output.is_empty() {
            eprint!("{stderr_output}");
        }
        return Err(DockerError::command_failed(&cmd_name, &stderr_output).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_builder_creation() {
        let builder = DockerComposeBuilder::new();
        assert_eq!(builder.files.len(), 1);
        assert_eq!(builder.files[0], "docker-compose.yml");
        assert!(builder.env_vars.is_empty());
        assert!(builder.services.is_empty());
    }

    #[test]
    fn test_docker_builder_add_file() {
        let mut builder = DockerComposeBuilder::new();
        builder.add_file("docker-compose.multi-l2.yml");

        assert_eq!(builder.files.len(), 2);
        assert_eq!(builder.files[1], "docker-compose.multi-l2.yml");
    }

    #[test]
    fn test_docker_builder_add_env() {
        let mut builder = DockerComposeBuilder::new();
        builder.add_env("TEST_KEY", "test_value");

        assert_eq!(builder.env_vars.len(), 1);
        assert_eq!(
            builder.env_vars.get("TEST_KEY"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_sandbox_config_mode_description() {
        assert_eq!(
            SandboxConfig::new(false, false).mode_description(),
            "local mode"
        );
        assert_eq!(
            SandboxConfig::new(true, false).mode_description(),
            "fork mode"
        );
        assert_eq!(
            SandboxConfig::new(false, true).mode_description(),
            "multi-L2 mode"
        );
        assert_eq!(
            SandboxConfig::new(true, true).mode_description(),
            "multi-L2 fork mode"
        );
    }

    #[test]
    fn test_docker_builder_build_up_command() {
        let mut builder = DockerComposeBuilder::new();
        builder.add_env("TEST_ENV", "test_value");

        let command = builder.build_up_command(true, false);
        let program = command.get_program();
        let args: Vec<&std::ffi::OsStr> = command.get_args().collect();

        // Test that we use either "docker" or "docker-compose" based on availability
        let (expected_program, base_args) = get_compose_command_parts();
        assert_eq!(program, expected_program);

        // Check for base args (e.g., "compose" for modern docker)
        for base_arg in base_args {
            assert!(args.contains(&std::ffi::OsStr::new(base_arg)));
        }

        assert!(args.contains(&std::ffi::OsStr::new("-f")));
        assert!(args.contains(&std::ffi::OsStr::new("docker-compose.yml")));
        assert!(args.contains(&std::ffi::OsStr::new("up")));
        assert!(args.contains(&std::ffi::OsStr::new("-d")));
    }

    #[test]
    fn test_compose_command_detection() {
        // Test that we get one of the expected commands
        let cmd = get_compose_command();
        assert!(cmd == "docker" || cmd == "docker-compose");

        // Test that command parts are consistent with detected command
        let (program, base_args) = get_compose_command_parts();
        if cmd == "docker" {
            assert_eq!(program, "docker");
            assert_eq!(base_args, vec!["compose"]);
        } else {
            assert_eq!(program, "docker-compose");
            assert!(base_args.is_empty());
        }
    }

    #[test]
    fn test_compose_command_name() {
        let name = get_compose_command_name();
        assert!(name == "docker compose" || name == "docker-compose");
    }
}
