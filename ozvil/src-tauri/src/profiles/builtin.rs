use crate::db::{
    models::{
        Action, ApprovalMode, ModeType, Profile, RestorePolicy, Trigger,
    },
    Database,
};
use crate::profiles::ProfileRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

pub fn seed_builtin_profiles(db: &Database) -> Result<()> {
    let repo = ProfileRepository::new(Arc::new(Database::open(
        &std::path::PathBuf::from(":memory:"),
    ).unwrap_or_else(|_| {
        // Fallback: use passed db reference directly via clone
        // This is a scaffold pattern; real impl passes Arc<Database>
        panic!("db unavailable for seeding")
    })));

    // Use direct SQL to seed builtin profiles rather than the repo
    // so we can reference the real db conn.
    let conn = db.conn.lock();

    let profiles = all_builtin_profiles();
    for p in &profiles {
        let existing: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM profiles WHERE id = ?1",
                rusqlite::params![p.id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if existing == 0 {
            conn.execute(
                "INSERT INTO profiles (id, name, mode_type, triggers, actions, restore_policy,
                  approval_mode, is_builtin, enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    p.id,
                    p.name,
                    serde_json::to_string(&p.mode_type).unwrap(),
                    serde_json::to_string(&p.triggers).unwrap(),
                    serde_json::to_string(&p.actions).unwrap(),
                    serde_json::to_string(&p.restore_policy).unwrap(),
                    serde_json::to_string(&p.approval_mode).unwrap(),
                    p.is_builtin as i32,
                    p.enabled as i32,
                    p.created_at.to_rfc3339(),
                    p.updated_at.to_rfc3339(),
                ],
            )?;
        }
    }

    Ok(())
}

pub fn all_builtin_profiles() -> Vec<Profile> {
    vec![
        render_mode(),
        studio_mode(),
        build_mode(),
        game_mode_plus(),
        design_mode(),
        recording_mode(),
    ]
}

fn base(id: &str, name: &str, mode_type: ModeType, restore: RestorePolicy) -> Profile {
    Profile {
        id: id.to_string(),
        name: name.to_string(),
        mode_type,
        triggers: vec![],
        actions: vec![],
        restore_policy: restore,
        approval_mode: ApprovalMode::AskFirst,
        is_builtin: true,
        enabled: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn render_mode() -> Profile {
    let mut p = base(
        "builtin-render",
        "Render Mode",
        ModeType::Render,
        RestorePolicy::OnAppQuit,
    );
    p.triggers = vec![
        Trigger::ProcessRunning { process_name: "Resolve.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Premiere Pro.exe".to_string() },
        Trigger::ProcessRunning { process_name: "AfterFX.exe".to_string() },
        Trigger::ProcessRunning { process_name: "blender.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Cinema 4D.exe".to_string() },
        Trigger::ProcessRunning { process_name: "adobe media encoder.exe".to_string() },
        Trigger::CpuAbove { process_name: None, percent: 60.0, duration_seconds: 120 },
        Trigger::ManualUi { profile_id: "builtin-render".to_string() },
    ];
    p.actions = vec![
        Action::PreventSleep,
        Action::SetPowerPlan { plan_id: "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c".to_string() },
        Action::WatchBattery { warn_below_percent: 20 },
        Action::WatchMemory { warn_above_percent: 85 },
        Action::WatchCpu { warn_above_percent: 90 },
    ];
    p
}

fn studio_mode() -> Profile {
    let mut p = base(
        "builtin-studio",
        "Studio Mode",
        ModeType::Studio,
        RestorePolicy::Manual,
    );
    p.triggers = vec![
        Trigger::ProcessRunning { process_name: "FL64.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Ableton Live.exe".to_string() },
        Trigger::ProcessRunning { process_name: "reaper.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Pro Tools.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Adobe Audition.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Cubase.exe".to_string() },
        Trigger::ManualUi { profile_id: "builtin-studio".to_string() },
    ];
    p.actions = vec![
        Action::PreventSleep,
        Action::ReduceInterruptions,
        Action::WatchMemory { warn_above_percent: 80 },
        Action::WatchCpu { warn_above_percent: 80 },
    ];
    p
}

fn build_mode() -> Profile {
    let mut p = base(
        "builtin-build",
        "Build Mode",
        ModeType::Build,
        RestorePolicy::OnResourceIdle,
    );
    p.triggers = vec![
        Trigger::ProcessRunning { process_name: "Docker Desktop.exe".to_string() },
        Trigger::ProcessRunning { process_name: "node.exe".to_string() },
        Trigger::ProcessRunning { process_name: "python.exe".to_string() },
        Trigger::ProcessRunning { process_name: "cargo.exe".to_string() },
        Trigger::ProcessRunning { process_name: "ollama.exe".to_string() },
        Trigger::ProcessRunning { process_name: "MSBuild.exe".to_string() },
        Trigger::ProcessRunning { process_name: "UnrealBuildTool.exe".to_string() },
        Trigger::CpuAbove { process_name: None, percent: 70.0, duration_seconds: 60 },
        Trigger::MemoryAbove { process_name: None, mb: 20480, duration_seconds: 60 },
        Trigger::ManualUi { profile_id: "builtin-build".to_string() },
    ];
    p.actions = vec![
        Action::PreventSleep,
        Action::WatchMemory { warn_above_percent: 85 },
        Action::WatchCpu { warn_above_percent: 90 },
        Action::WatchBattery { warn_below_percent: 20 },
    ];
    p
}

fn game_mode_plus() -> Profile {
    let mut p = base(
        "builtin-game",
        "Game Mode Plus",
        ModeType::Game,
        RestorePolicy::OnAppQuit,
    );
    p.triggers = vec![
        Trigger::ProcessRunning { process_name: "steam.exe".to_string() },
        Trigger::ProcessRunning { process_name: "EpicGamesLauncher.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Unity.exe".to_string() },
        Trigger::ProcessRunning { process_name: "UE4Editor.exe".to_string() },
        Trigger::ProcessRunning { process_name: "obs64.exe".to_string() },
        Trigger::ManualUi { profile_id: "builtin-game".to_string() },
    ];
    p.actions = vec![
        Action::PreventSleep,
        Action::SetPowerPlan { plan_id: "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c".to_string() },
        Action::ReduceInterruptions,
        Action::WatchMemory { warn_above_percent: 85 },
        Action::WatchCpu { warn_above_percent: 90 },
    ];
    p
}

fn design_mode() -> Profile {
    let mut p = base(
        "builtin-design",
        "Design Mode",
        ModeType::Design,
        RestorePolicy::OnAppQuit,
    );
    p.triggers = vec![
        Trigger::ProcessRunning { process_name: "Photoshop.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Illustrator.exe".to_string() },
        Trigger::ProcessRunning { process_name: "InDesign.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Figma.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Affinity Designer.exe".to_string() },
        Trigger::ProcessRunning { process_name: "webui.py".to_string() },
        Trigger::ProcessRunning { process_name: "comfyui.py".to_string() },
        Trigger::ManualUi { profile_id: "builtin-design".to_string() },
    ];
    p.actions = vec![
        Action::ReduceInterruptions,
        Action::WatchMemory { warn_above_percent: 85 },
        Action::WatchCpu { warn_above_percent: 85 },
    ];
    p
}

fn recording_mode() -> Profile {
    let mut p = base(
        "builtin-recording",
        "Recording Mode",
        ModeType::Recording,
        RestorePolicy::OnAppQuit,
    );
    p.triggers = vec![
        Trigger::ProcessRunning { process_name: "obs64.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Streamlabs OBS.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Camtasia.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Zoom.exe".to_string() },
        Trigger::ProcessRunning { process_name: "Teams.exe".to_string() },
        Trigger::ProcessRunning { process_name: "POWERPNT.EXE".to_string() },
        Trigger::ManualUi { profile_id: "builtin-recording".to_string() },
    ];
    p.actions = vec![
        Action::PreventSleep,
        Action::ReduceInterruptions,
        Action::WatchBattery { warn_below_percent: 25 },
        Action::WatchMemory { warn_above_percent: 80 },
        Action::WatchCpu { warn_above_percent: 80 },
    ];
    p
}
