#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use ozvil_lib::cli::{Cli, CliSubcommand};
use ozvil_lib::core::AppState;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_safe_mode = args.contains(&"--safe-mode".to_string())
        || args.contains(&"safe-start".to_string());

    // If first arg after binary is a known CLI subcommand, run in CLI mode
    let is_cli = args.len() > 1
        && !args[1].starts_with("--safe-mode")
        && matches!(
            args[1].as_str(),
            "status" | "profiles" | "start" | "stop" | "restore" | "dry-run" | "logs"
                | "profile" | "safe-start"
        );

    if is_cli {
        let cli = Cli::parse();
        ozvil_lib::cli::run_cli(cli);
        return;
    }

    ozvil_lib::run_gui(is_safe_mode);
}
