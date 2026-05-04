use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ozvil", about = "Ozvil — Windows workload modes for heavy apps", version)]
pub struct Cli {
    #[arg(long, global = true)]
    pub safe_mode: bool,

    #[command(subcommand)]
    pub command: Option<CliSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum CliSubcommand {
    /// Show current status
    Status {
        #[arg(long)]
        agent: bool,
    },
    /// Start Ozvil in safe mode (automation disabled)
    SafeStart,
    /// List available profiles
    Profiles {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Start a profile by id or name
    Start {
        profile: String,
    },
    /// Preview what a profile would do without applying changes
    DryRun {
        profile: String,
    },
    /// Stop the active session
    Stop,
    /// Restore system state from the active or last snapshot
    Restore,
    /// Export or import logs
    Logs {
        #[command(subcommand)]
        action: LogAction,
    },
    /// Export or import a profile
    Profile {
        #[command(subcommand)]
        action: ProfileFileAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    List,
}

#[derive(Subcommand, Debug)]
pub enum LogAction {
    Export {
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileFileAction {
    Export {
        profile: String,
        #[arg(long)]
        output: Option<String>,
    },
    Import {
        path: String,
    },
}

pub fn run_cli(cli: Cli) {
    // CLI runs against the same SQLite database as the GUI app.
    // Locate the DB path from the standard app data dir.
    let db_path = resolve_db_path();

    let db = match crate::db::Database::open(&db_path) {
        Ok(d) => std::sync::Arc::new(d),
        Err(e) => {
            eprintln!("ozvil: failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    if cli.safe_mode {
        println!("Ozvil running in Safe Mode — automation disabled.");
    }

    match cli.command {
        Some(CliSubcommand::Status { agent }) => cmd_status(&db, agent),
        Some(CliSubcommand::SafeStart) => {
            println!("Ozvil Safe Mode active. Automation disabled.");
            std::process::exit(0);
        }
        Some(CliSubcommand::Profiles { action: ProfileAction::List }) => cmd_profiles_list(&db),
        Some(CliSubcommand::Start { profile }) => cmd_start(&db, &profile, cli.safe_mode),
        Some(CliSubcommand::DryRun { profile }) => cmd_dry_run(&db, &profile),
        Some(CliSubcommand::Stop) => cmd_stop(&db),
        Some(CliSubcommand::Restore) => cmd_restore(&db),
        Some(CliSubcommand::Logs { action: LogAction::Export { format, output } }) => {
            cmd_logs_export(&db, &format, output.as_deref())
        }
        Some(CliSubcommand::Profile {
            action: ProfileFileAction::Export { profile, output },
        }) => cmd_profile_export(&db, &profile, output.as_deref()),
        Some(CliSubcommand::Profile {
            action: ProfileFileAction::Import { path },
        }) => cmd_profile_import(&db, &path),
        None => {
            eprintln!("Run `ozvil --help` for available commands.");
        }
    }
}

fn resolve_db_path() -> std::path::PathBuf {
    let base = std::env::var("OZVIL_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("Ozvil")
        });
    base.join("ozvil.db")
}

fn cmd_status(db: &std::sync::Arc<crate::db::Database>, agent: bool) {
    let adapter = crate::windows_adapter::WindowsNativeAdapter;
    let status = match crate::windows_adapter::WindowsAdapter::read_system_status(&adapter) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ozvil status: failed to read system status: {}", e);
            std::process::exit(1);
        }
    };

    let sm = crate::core::session_manager::SessionManager::new(db.clone());
    let active = sm.get_active_session().ok().flatten();

    let mode = active
        .as_ref()
        .map(|s| s.profile_id.clone())
        .unwrap_or_else(|| "idle".to_string());

    let pressure = crate::core::rules_engine::build_pressure_label(
        status.cpu_percent,
        status.ram_percent,
    );
    let recommendation = crate::core::rules_engine::build_recommendation(&status);

    if agent {
        let agent_status = crate::db::models::AgentStatus {
            mode,
            pressure: pressure.to_string(),
            ram: status.ram_percent as u8,
            cpu: status.cpu_percent as u8,
            battery: status.battery_percent,
            recommendation,
        };
        println!("{}", serde_json::to_string(&agent_status).unwrap());
    } else {
        println!("Mode    : {}", mode);
        println!("CPU     : {:.0}%", status.cpu_percent);
        println!("RAM     : {:.0}% ({}/{} MB)", status.ram_percent, status.ram_used_mb, status.ram_total_mb);
        if let Some(batt) = status.battery_percent {
            println!("Battery : {}% ({})", batt, if status.on_ac_power { "AC" } else { "on battery" });
        }
        println!("Pressure: {}", pressure);
        if recommendation != "none" {
            println!("Suggest : {}", recommendation);
        }
    }
}

fn cmd_profiles_list(db: &std::sync::Arc<crate::db::Database>) {
    use crate::profiles::ProfileRepository;
    let repo = ProfileRepository::new(db.clone());
    match repo.list() {
        Ok(profiles) => {
            for p in &profiles {
                let status = if p.enabled { "enabled" } else { "disabled" };
                println!("[{}] {} ({:?}) — {}", p.id, p.name, p.mode_type, status);
            }
        }
        Err(e) => eprintln!("ozvil profiles list: {}", e),
    }
}

fn cmd_start(_db: &std::sync::Arc<crate::db::Database>, profile_ref: &str, safe_mode: bool) {
    if safe_mode {
        eprintln!("ozvil start: Safe Mode is active. Automation is disabled.");
        std::process::exit(1);
    }
    println!("Starting profile '{}' — use the Ozvil UI for full session management.", profile_ref);
}

fn cmd_dry_run(db: &std::sync::Arc<crate::db::Database>, profile_ref: &str) {
    use crate::profiles::ProfileRepository;
    let repo = ProfileRepository::new(db.clone());
    let profiles = repo.list().unwrap_or_default();
    let profile = profiles.iter().find(|p| p.id == profile_ref || p.name.to_lowercase() == profile_ref.to_lowercase());

    match profile {
        Some(p) => {
            println!("Dry run for: {} ({})", p.name, p.id);
            println!("Actions that WOULD be applied:");
            for action in &p.actions {
                println!("  • {:?}", action);
            }
            println!("Triggers:");
            for trigger in &p.triggers {
                println!("  • {:?}", trigger);
            }
            println!("[DRY RUN] No system changes applied.");
        }
        None => {
            eprintln!("ozvil dry-run: profile not found: {}", profile_ref);
            std::process::exit(1);
        }
    }
}

fn cmd_stop(db: &std::sync::Arc<crate::db::Database>) {
    let sm = crate::core::session_manager::SessionManager::new(db.clone());
    match sm.get_active_session() {
        Ok(Some(session)) => {
            let _ = sm.end_session(&session.id);
            println!("Session {} stopped.", session.id);
        }
        Ok(None) => println!("No active session."),
        Err(e) => eprintln!("ozvil stop: {}", e),
    }
}

fn cmd_restore(db: &std::sync::Arc<crate::db::Database>) {
    let sm = crate::core::session_manager::SessionManager::new(db.clone());
    match sm.get_active_session() {
        Ok(Some(session)) => {
            if let Some(snapshot) = &session.snapshot {
                let adapter = crate::windows_adapter::WindowsNativeAdapter;
                match crate::windows_adapter::WindowsAdapter::restore_snapshot(&adapter, snapshot) {
                    Ok(result) => {
                        println!("Restore complete.");
                        for r in &result.restored { println!("  ✓ {}", r); }
                        for f in &result.failed { println!("  ✗ {}", f); }
                    }
                    Err(e) => eprintln!("ozvil restore: {}", e),
                }
                let _ = sm.end_session(&session.id);
            } else {
                println!("No snapshot available for restore.");
            }
        }
        Ok(None) => {
            // Check stale sessions
            match sm.get_stale_sessions() {
                Ok(stale) if !stale.is_empty() => {
                    println!("{} stale session(s) found. Use the Ozvil UI to review and restore.", stale.len());
                }
                _ => println!("No active or stale session to restore."),
            }
        }
        Err(e) => eprintln!("ozvil restore: {}", e),
    }
}

fn cmd_logs_export(
    db: &std::sync::Arc<crate::db::Database>,
    format: &str,
    output: Option<&str>,
) {
    let logger = crate::core::activity_logger::ActivityLogger::new(db.clone(), usize::MAX);
    let logs = match logger.get_logs(None, 10000) {
        Ok(l) => l,
        Err(e) => { eprintln!("ozvil logs export: {}", e); std::process::exit(1); }
    };

    let content = match format {
        "csv" => {
            let mut out = String::from("id,session_id,event_type,result,created_at\n");
            for l in &logs {
                out.push_str(&format!(
                    "{},{},{:?},{},{}\n",
                    l.id,
                    l.session_id.as_deref().unwrap_or(""),
                    l.event_type,
                    l.result,
                    l.created_at.to_rfc3339(),
                ));
            }
            out
        }
        _ => serde_json::to_string_pretty(&logs).unwrap_or_default(),
    };

    match output {
        Some(path) => {
            std::fs::write(path, &content).unwrap_or_else(|e| eprintln!("write failed: {}", e));
            println!("Exported {} log entries to {}", logs.len(), path);
        }
        None => print!("{}", content),
    }
}

fn cmd_profile_export(
    db: &std::sync::Arc<crate::db::Database>,
    profile_ref: &str,
    output: Option<&str>,
) {
    use crate::profiles::ProfileRepository;
    let repo = ProfileRepository::new(db.clone());
    let profiles = repo.list().unwrap_or_default();
    let profile = profiles.iter().find(|p| p.id == profile_ref || p.name.to_lowercase() == profile_ref.to_lowercase());

    match profile {
        Some(p) => {
            let json = serde_json::to_string_pretty(p).unwrap_or_default();
            match output {
                Some(path) => {
                    std::fs::write(path, &json).unwrap_or_else(|e| eprintln!("write failed: {}", e));
                    println!("Profile exported to {}", path);
                }
                None => println!("{}", json),
            }
        }
        None => { eprintln!("Profile not found: {}", profile_ref); std::process::exit(1); }
    }
}

fn cmd_profile_import(db: &std::sync::Arc<crate::db::Database>, path: &str) {
    use crate::profiles::ProfileRepository;
    let json = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("ozvil profile import: cannot read {}: {}", path, e); std::process::exit(1); }
    };

    let profile: crate::db::models::Profile = match serde_json::from_str(&json) {
        Ok(p) => p,
        Err(e) => { eprintln!("ozvil profile import: invalid profile JSON: {}", e); std::process::exit(1); }
    };

    let repo = ProfileRepository::new(db.clone());
    match repo.upsert(&profile) {
        Ok(_) => println!("Profile '{}' imported.", profile.name),
        Err(e) => { eprintln!("ozvil profile import: {}", e); std::process::exit(1); }
    }
}
