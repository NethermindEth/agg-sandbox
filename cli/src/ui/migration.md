# Migration Guide: Using the New UI Module

This guide shows how to migrate from direct `println!` calls and scattered UI patterns to the new unified UI module.

## Quick Start

### Initialize UI in main()
```rust
use ui::{OutputFormat, init_ui};

// In main function:
init_ui(OutputFormat::Human, cli.quiet);
```

### Basic Usage

#### Before
```rust
println!("✓ Operation successful");
eprintln!("✗ Error occurred");
println!("{} Warning message", "⚠".yellow());
```

#### After
```rust
use crate::ui;

ui::ui().success("Operation successful");
ui::ui().error("Error occurred");
ui::ui().warning("Warning message");

// Or use macros:
ui_success!("Operation successful");
ui_error!("Error occurred");
ui_warning!("Warning message");
```

### Table Formatting

#### Before
```rust
use crate::commands::bridge::common::table;

table::print_table("Bridge Information", &[
    ("Network", "Ethereum L1"),
    ("Status", "Active"),
]);
```

#### After
```rust
use crate::ui;

ui::ui().table("🌉 Bridge Information", &[
    ("Network", "Ethereum L1"),
    ("Status", "Active"),
]);
```

### JSON Output

#### Before
```rust
use crate::api;

if json_mode {
    api::print_raw_json(&data);
} else {
    api::print_json_response("Bridge Data", &data);
}
```

#### After
```rust
use crate::ui::{UI, OutputFormat};

let ui_instance = UI::new(if json_mode { 
    OutputFormat::Json 
} else { 
    OutputFormat::Human 
});

ui_instance.data("Bridge Data", &data);
```

### Progress Messages

#### Before
```rust
use crate::progress;

progress::ProgressManager::info("Processing request...");
```

#### After
```rust
ui::ui().info("Processing request...");
ui::ui().println(&crate::ui::ProgressMessage::with_progress(
    "Processing request", 3, 5
));
```

### Complex Output Building

```rust
use crate::ui::{OutputBuilder, UI, OutputFormat};

let ui_instance = UI::new(OutputFormat::Human);
OutputBuilder::new(ui_instance)
    .success("Bridge operation completed")
    .table("Results", &[
        ("Transaction Hash", "0x123..."),
        ("Gas Used", "21000"),
    ])
    .blank_line()
    .tip("Wait 5 seconds before claiming")
    .build();
```

## Migration Strategy

1. **Phase 1**: Add UI module alongside existing patterns
2. **Phase 2**: Update one module at a time (start with new features)
3. **Phase 3**: Replace existing table/json utilities with compat layer
4. **Phase 4**: Remove old printing patterns

## Backward Compatibility

The UI module provides compatibility functions in `ui::compat`:

```rust
use crate::ui::output::compat;

// These work exactly like before but use the new UI system internally:
compat::print_json_response("Title", &data);
compat::print_raw_json(&data);
compat::print_table("Title", &rows);
```

## Command-Level JSON Support

For commands that support `--json` flag:

```rust
pub async fn handle_command(args: CommandArgs) -> Result<()> {
    let ui_instance = UI::new(if args.json {
        OutputFormat::Json
    } else {
        OutputFormat::Human  
    });

    // Use ui_instance for all output in this command
    ui_instance.success("Command completed");
    
    // Or temporarily override global UI for this command
    // (Advanced usage - be careful with thread safety)
}
```

## Benefits

- **Consistency**: All output goes through the same formatting system
- **Testability**: UI behavior can be easily tested
- **JSON Support**: Automatic JSON formatting for all message types
- **Maintainability**: Centralized styling and formatting logic
- **Extensibility**: Easy to add new message types and formatters