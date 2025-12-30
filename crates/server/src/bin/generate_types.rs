//! Run with: cargo run --package server --bin generate-types --features typescript

use std::fs;
use std::path::Path;

fn main() {
    println!("Generating TypeScript types...");

    let out_dir = Path::new("frontend/src/types/generated");

    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    #[cfg(feature = "typescript")]
    {
        use ts_rs::TS;

        opencode_core::TaskStatus::export_all_to(out_dir).expect("Failed to export TaskStatus");
        opencode_core::Task::export_all_to(out_dir).expect("Failed to export Task");
        opencode_core::CreateTaskRequest::export_all_to(out_dir)
            .expect("Failed to export CreateTaskRequest");
        opencode_core::UpdateTaskRequest::export_all_to(out_dir)
            .expect("Failed to export UpdateTaskRequest");
        opencode_core::SessionPhase::export_all_to(out_dir)
            .expect("Failed to export SessionPhase");
        opencode_core::SessionStatus::export_all_to(out_dir)
            .expect("Failed to export SessionStatus");
        opencode_core::Session::export_all_to(out_dir).expect("Failed to export Session");

        events::EventEnvelope::export_all_to(out_dir).expect("Failed to export EventEnvelope");
        events::Event::export_all_to(out_dir).expect("Failed to export Event");
        events::AgentMessageData::export_all_to(out_dir)
            .expect("Failed to export AgentMessageData");
        events::ToolExecutionData::export_all_to(out_dir)
            .expect("Failed to export ToolExecutionData");

        vcs::Workspace::export_all_to(out_dir).expect("Failed to export Workspace");
        vcs::WorkspaceStatus::export_all_to(out_dir).expect("Failed to export WorkspaceStatus");
        vcs::MergeResult::export_all_to(out_dir).expect("Failed to export MergeResult");
        vcs::ConflictFile::export_all_to(out_dir).expect("Failed to export ConflictFile");
        vcs::ConflictType::export_all_to(out_dir).expect("Failed to export ConflictType");

        websocket::ClientMessage::export_all_to(out_dir).expect("Failed to export ClientMessage");
        websocket::ServerMessage::export_all_to(out_dir).expect("Failed to export ServerMessage");
        websocket::SubscriptionFilter::export_all_to(out_dir)
            .expect("Failed to export SubscriptionFilter");

        server::routes::TransitionRequest::export_all_to(out_dir)
            .expect("Failed to export TransitionRequest");
        server::routes::TransitionResponse::export_all_to(out_dir)
            .expect("Failed to export TransitionResponse");
        server::routes::ExecuteResponse::export_all_to(out_dir)
            .expect("Failed to export ExecuteResponse");
        server::routes::PhaseResultDto::export_all_to(out_dir)
            .expect("Failed to export PhaseResultDto");
        server::routes::WorkspaceResponse::export_all_to(out_dir)
            .expect("Failed to export WorkspaceResponse");
        server::routes::DiffResponse::export_all_to(out_dir)
            .expect("Failed to export DiffResponse");
        server::routes::MergeRequest::export_all_to(out_dir)
            .expect("Failed to export MergeRequest");
        server::routes::MergeResponse::export_all_to(out_dir)
            .expect("Failed to export MergeResponse");

        println!("Types exported to {}", out_dir.display());

        generate_index(out_dir);
    }

    #[cfg(not(feature = "typescript"))]
    {
        eprintln!("Error: typescript feature is not enabled");
        eprintln!("Run with: cargo run --package server --bin generate-types --features typescript");
        std::process::exit(1);
    }
}

#[cfg(feature = "typescript")]
fn generate_index(out_dir: &Path) {
    use std::io::Write;

    let index_path = out_dir.join("index.ts");
    let mut file = fs::File::create(&index_path).expect("Failed to create index.ts");

    let exports = r#"// Auto-generated - regenerate with: cargo run --package server --bin generate-types --features typescript

export * from './Task';
export * from './TaskStatus';
export * from './CreateTaskRequest';
export * from './UpdateTaskRequest';
export * from './Session';
export * from './SessionPhase';
export * from './SessionStatus';

export * from './EventEnvelope';
export * from './Event';
export * from './AgentMessageData';
export * from './ToolExecutionData';

export * from './Workspace';
export * from './WorkspaceStatus';
export * from './MergeResult';
export * from './ConflictFile';
export * from './ConflictType';

export * from './ClientMessage';
export * from './ServerMessage';
export * from './SubscriptionFilter';

export * from './TransitionRequest';
export * from './TransitionResponse';
export * from './ExecuteResponse';
export * from './PhaseResultDto';
export * from './WorkspaceResponse';
export * from './DiffResponse';
export * from './MergeRequest';
export * from './MergeResponse';
"#;

    file.write_all(exports.as_bytes())
        .expect("Failed to write index.ts");

    println!("Generated {}", index_path.display());
}
