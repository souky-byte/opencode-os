use axum_test::TestServer;
use serde_json::{json, Value};
use server::{create_router, state::AppState};
use std::process::Command;
use std::sync::Mutex;
use tempfile::TempDir;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn create_temp_git_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo with main branch (git 2.28+)
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git name");

    std::fs::write(temp_dir.path().join("README.md"), "# Test\n").expect("Failed to write README");

    Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to git add");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to git commit");

    temp_dir
}

async fn setup_test_server() -> (TestServer, TempDir, MockServer, std::sync::MutexGuard<'static, ()>) {
    let lock = TEST_MUTEX.lock().unwrap();
    
    let temp_dir = create_temp_git_repo();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite:{}", db_path.display());

    let pool = db::create_pool(&db_url).await.expect("Failed to create pool");
    db::run_migrations(&pool).await.expect("Failed to run migrations");

    let mock_opencode = MockServer::start().await;

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let state = AppState::new(pool, &mock_opencode.uri());
    let app = create_router(state);

    let server = TestServer::new(app).expect("Failed to create test server");

    (server, temp_dir, mock_opencode, lock)
}

mod health {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let response = server.get("/health").await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert_eq!(body["status"], "ok");
    }
}

mod tasks_crud {
    use super::*;

    #[tokio::test]
    async fn test_create_task_returns_201_created() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Test Task",
                "description": "Test Description"
            }))
            .await;

        response.assert_status(axum::http::StatusCode::CREATED);
        let body: Value = response.json();
        assert_eq!(body["title"], "Test Task");
        assert_eq!(body["description"], "Test Description");
        assert_eq!(body["status"], "todo");
        assert!(body["id"].is_string());
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        server
            .post("/api/tasks")
            .json(&json!({
                "title": "Task 1",
                "description": "Desc 1"
            }))
            .await;

        server
            .post("/api/tasks")
            .json(&json!({
                "title": "Task 2",
                "description": "Desc 2"
            }))
            .await;

        let response = server.get("/api/tasks").await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_task() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Get Task Test",
                "description": "Description"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let response = server.get(&format!("/api/tasks/{}", task_id)).await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert_eq!(body["title"], "Get Task Test");
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let fake_id = uuid::Uuid::new_v4();
        let response = server.get(&format!("/api/tasks/{}", fake_id)).await;

        response.assert_status_not_found();
    }
}

mod e2e_jj_workspace {
    use super::*;

    fn create_temp_jj_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        Command::new("jj")
            .args(["git", "init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init jj repo");

        Command::new("jj")
            .args(["config", "set", "--repo", "user.email", "test@test.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set jj email");

        Command::new("jj")
            .args(["config", "set", "--repo", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set jj name");

        std::fs::write(temp_dir.path().join("README.md"), "# Test Project\n")
            .expect("Failed to write README");

        Command::new("jj")
            .args(["bookmark", "create", "main", "-r", "@"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create main bookmark");

        Command::new("jj")
            .args(["new"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create new change");

        temp_dir
    }

    async fn setup_e2e_server() -> (TestServer, TempDir, std::sync::MutexGuard<'static, ()>) {
        let lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let temp_dir = create_temp_jj_repo();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = db::create_pool(&db_url).await.expect("Failed to create pool");
        db::run_migrations(&pool).await.expect("Failed to run migrations");

        std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

        let state = AppState::new(pool, "http://localhost:9999");
        let app = create_router(state);

        let server = TestServer::new(app).expect("Failed to create test server");

        (server, temp_dir, lock)
    }

    #[tokio::test]
    async fn test_e2e_create_jj_workspace() {
        let (server, temp_dir, _lock) = setup_e2e_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "E2E JJ Workspace Test",
                "description": "Test jj workspace creation"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let workspace_response = server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        workspace_response.assert_status(axum::http::StatusCode::CREATED);
        let workspace: Value = workspace_response.json();

        assert_eq!(workspace["task_id"], task_id);

        let workspace_path = workspace["path"].as_str().unwrap();
        assert!(
            std::path::Path::new(workspace_path).exists(),
            "Workspace directory should exist at {}",
            workspace_path
        );

        let jj_output = Command::new("jj")
            .args(["workspace", "list"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to list jj workspaces");

        let output = String::from_utf8_lossy(&jj_output.stdout);
        assert!(
            output.contains(&format!("task-{}", task_id)),
            "jj workspace list should contain task workspace: {}",
            output
        );
    }

    #[tokio::test]
    async fn test_e2e_workspace_diff_with_real_changes() {
        let (server, _temp_dir, _lock) = setup_e2e_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "E2E Diff Test",
                "description": "Test workspace diff with real file changes"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let workspace_response = server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let workspace: Value = workspace_response.json();
        let workspace_path = workspace["path"].as_str().unwrap();

        let test_file = std::path::Path::new(workspace_path).join("test_change.txt");
        std::fs::write(&test_file, "Hello from E2E test!\n").expect("Failed to write test file");

        let diff_response = server
            .get(&format!("/api/workspaces/{}/diff", task_id))
            .await;

        diff_response.assert_status_ok();
        let diff: Value = diff_response.json();

        let diff_content = diff["diff"].as_str().unwrap();
        assert!(
            diff_content.contains("test_change.txt"),
            "Diff should contain the new file name. Got: {}",
            diff_content
        );
        assert!(
            diff_content.contains("Hello from E2E test!"),
            "Diff should contain the file content. Got: {}",
            diff_content
        );
    }

    #[tokio::test]
    async fn test_e2e_workspace_status_shows_changes() {
        let (server, _temp_dir, _lock) = setup_e2e_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "E2E Status Test",
                "description": "Test workspace status with file changes"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let workspace_response = server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let workspace: Value = workspace_response.json();
        let workspace_path = workspace["path"].as_str().unwrap();

        let test_file = std::path::Path::new(workspace_path).join("new_file.rs");
        std::fs::write(&test_file, "fn main() {}\n").expect("Failed to write test file");

        let status_response = server
            .get(&format!("/api/workspaces/{}", task_id))
            .await;

        status_response.assert_status_ok();
        let status: Value = status_response.json();

        let status_content = status["status"].as_str().unwrap();
        assert!(
            status_content.contains("new_file.rs"),
            "Status should show the new file. Got: {}",
            status_content
        );
    }

    #[tokio::test]
    async fn test_e2e_multiple_workspaces() {
        let (server, temp_dir, _lock) = setup_e2e_server().await;

        let mut task_ids = Vec::new();

        for i in 1..=3 {
            let create_response = server
                .post("/api/tasks")
                .json(&json!({
                    "title": format!("Multi Workspace Task {}", i),
                    "description": format!("Task {} for multiple workspace test", i)
                }))
                .await;

            let created: Value = create_response.json();
            let task_id = created["id"].as_str().unwrap().to_string();

            server
                .post(&format!("/api/tasks/{}/workspace", task_id))
                .await
                .assert_status(axum::http::StatusCode::CREATED);

            task_ids.push(task_id);
        }

        let list_response = server.get("/api/workspaces").await;
        list_response.assert_status_ok();

        let workspaces: Value = list_response.json();
        let workspaces_arr = workspaces.as_array().unwrap();

        assert_eq!(
            workspaces_arr.len(),
            3,
            "Should have 3 workspaces, got {}",
            workspaces_arr.len()
        );

        let jj_output = Command::new("jj")
            .args(["workspace", "list"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to list jj workspaces");

        let output = String::from_utf8_lossy(&jj_output.stdout);

        for task_id in &task_ids {
            assert!(
                output.contains(&format!("task-{}", task_id)),
                "jj should list workspace for task {}",
                task_id
            );
        }
    }

    #[tokio::test]
    async fn test_e2e_workspace_cleanup() {
        let (server, temp_dir, _lock) = setup_e2e_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "E2E Cleanup Test",
                "description": "Test workspace cleanup"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let workspace_response = server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let workspace: Value = workspace_response.json();
        let workspace_path = workspace["path"].as_str().unwrap().to_string();

        assert!(
            std::path::Path::new(&workspace_path).exists(),
            "Workspace should exist before cleanup"
        );

        let delete_response = server
            .delete(&format!("/api/workspaces/{}", task_id))
            .await;

        delete_response.assert_status(axum::http::StatusCode::NO_CONTENT);

        assert!(
            !std::path::Path::new(&workspace_path).exists(),
            "Workspace directory should be removed after cleanup"
        );

        let jj_output = Command::new("jj")
            .args(["workspace", "list"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to list jj workspaces");

        let output = String::from_utf8_lossy(&jj_output.stdout);
        let workspace_name = format!("task-{}:", task_id);
        let workspace_still_exists = output
            .lines()
            .any(|line| line.starts_with(&workspace_name));
        assert!(
            !workspace_still_exists,
            "jj workspace list should not contain deleted workspace"
        );
    }
}

mod task_transitions {
    use super::*;

    #[tokio::test]
    async fn test_transition_todo_to_planning() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Transition Test",
                "description": "Testing transitions"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let response = server
            .post(&format!("/api/tasks/{}/transition", task_id))
            .json(&json!({
                "status": "planning"
            }))
            .await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert_eq!(body["task"]["status"], "planning");
        assert_eq!(body["previous_status"], "todo");
    }

    #[tokio::test]
    async fn test_invalid_transition_todo_to_done() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Invalid Transition Test",
                "description": "Testing invalid transitions"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let response = server
            .post(&format!("/api/tasks/{}/transition", task_id))
            .json(&json!({
                "status": "done"
            }))
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_full_state_machine_path() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Full Path Test",
                "description": "Testing full state machine"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let transitions = vec![
            "planning",
            "planning_review",
            "in_progress",
            "ai_review",
            "review",
            "done",
        ];

        for status in transitions {
            let response = server
                .post(&format!("/api/tasks/{}/transition", task_id))
                .json(&json!({ "status": status }))
                .await;

            response.assert_status_ok();
            let body: Value = response.json();
            assert_eq!(body["task"]["status"], status);
        }
    }
}

mod task_execute {
    use super::*;

    #[tokio::test]
    async fn test_execute_planning_phase() {
        let (server, _temp_dir, mock_opencode, _lock) = setup_test_server().await;

        Mock::given(method("POST"))
            .and(path("/session"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "mock-session-123",
                "title": "Planning: Test Task",
                "parent_id": null,
                "created_at": "2025-12-30T12:00:00Z"
            })))
            .mount(&mock_opencode)
            .await;

        Mock::given(method("POST"))
            .and(path_regex(r"/session/.*/message"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "session_id": "mock-session-123",
                "message": {
                    "id": "msg-123",
                    "role": "assistant",
                    "content": "# Plan\n\n1. Step one\n2. Step two",
                    "created_at": "2025-12-30T12:00:00Z"
                }
            })))
            .mount(&mock_opencode)
            .await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Execute Test",
                "description": "Testing execute"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let transition_response = server
            .post(&format!("/api/tasks/{}/transition", task_id))
            .json(&json!({ "status": "planning" }))
            .await;
        transition_response.assert_status_ok();

        let execute_response = server
            .post(&format!("/api/tasks/{}/execute", task_id))
            .await;

        execute_response.assert_status_ok();
        let body: Value = execute_response.json();
        assert!(
            body["result"]["plan_path"].is_string() || body["result"]["session_id"].is_string(),
            "Expected plan_path or session_id in response: {:?}",
            body
        );
    }
}

mod sessions {
    use super::*;

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let response = server.get("/api/sessions").await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn test_list_sessions_for_task() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Sessions Test",
                "description": "Testing sessions"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let response = server
            .get(&format!("/api/tasks/{}/sessions", task_id))
            .await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert!(body.is_array());
    }
}

mod workspaces {
    use super::*;

    #[tokio::test]
    async fn test_list_workspaces_empty() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let response = server.get("/api/workspaces").await;

        response.assert_status_ok();
        let body: Value = response.json();
        assert!(body.is_array());
        assert!(body.as_array().unwrap().is_empty());
    }
}

mod kanban_flow {
    use super::*;

    fn mock_opencode_session_and_message(mock: &MockServer) -> impl std::future::Future<Output = ()> + '_ {
        async move {
            Mock::given(method("POST"))
                .and(path("/session"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "flow-session-123",
                    "title": "Session",
                    "parent_id": null,
                    "created_at": "2025-12-30T12:00:00Z"
                })))
                .mount(mock)
                .await;

            Mock::given(method("POST"))
                .and(path_regex(r"/session/.*/message"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "session_id": "flow-session-123",
                    "message": {
                        "id": "msg-flow-123",
                        "role": "assistant",
                        "content": "# Plan\n\n## Steps\n1. Implement feature\n2. Write tests\n3. Document",
                        "created_at": "2025-12-30T12:00:00Z"
                    }
                })))
                .mount(mock)
                .await;
        }
    }

    #[tokio::test]
    async fn test_planning_phase_creates_plan_file() {
        let (server, _temp_dir, mock_opencode, _lock) = setup_test_server().await;
        mock_opencode_session_and_message(&mock_opencode).await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Plan File Test",
                "description": "Test that planning creates a plan file"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/transition", task_id))
            .json(&json!({ "status": "planning" }))
            .await;

        let execute_response = server
            .post(&format!("/api/tasks/{}/execute", task_id))
            .await;

        execute_response.assert_status_ok();
        let body: Value = execute_response.json();

        assert!(body["result"]["plan_path"].is_string());
        let plan_path = body["result"]["plan_path"].as_str().unwrap();
        
        let full_path = std::path::Path::new(plan_path);
        assert!(full_path.exists(), "Plan file should exist at {:?}", full_path);

        let task_response = server.get(&format!("/api/tasks/{}", task_id)).await;
        let task: Value = task_response.json();
        assert_eq!(task["status"], "planning_review");
    }

    #[tokio::test]
    async fn test_planning_phase_creates_session_in_db() {
        let (server, _temp_dir, mock_opencode, _lock) = setup_test_server().await;
        mock_opencode_session_and_message(&mock_opencode).await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Session DB Test",
                "description": "Test that planning creates a session in DB"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/transition", task_id))
            .json(&json!({ "status": "planning" }))
            .await;

        server
            .post(&format!("/api/tasks/{}/execute", task_id))
            .await;

        let sessions_response = server
            .get(&format!("/api/tasks/{}/sessions", task_id))
            .await;

        sessions_response.assert_status_ok();
        let sessions: Value = sessions_response.json();
        
        assert!(sessions.is_array());
        let sessions_arr = sessions.as_array().unwrap();
        assert!(!sessions_arr.is_empty(), "Should have at least one session");
        
        let session = &sessions_arr[0];
        assert_eq!(session["phase"], "planning");
        assert_eq!(session["status"], "completed");
    }

    #[tokio::test]
    async fn test_execute_from_todo_auto_transitions_to_planning() {
        let (server, _temp_dir, mock_opencode, _lock) = setup_test_server().await;
        mock_opencode_session_and_message(&mock_opencode).await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Auto Transition Test",
                "description": "Test execute from TODO auto-transitions"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();
        assert_eq!(created["status"], "todo");

        let execute_response = server
            .post(&format!("/api/tasks/{}/execute", task_id))
            .await;

        execute_response.assert_status_ok();

        let task_response = server.get(&format!("/api/tasks/{}", task_id)).await;
        let task: Value = task_response.json();
        assert_eq!(task["status"], "planning_review");
    }

    #[tokio::test]
    async fn test_planning_review_awaits_approval() {
        let (server, _temp_dir, mock_opencode, _lock) = setup_test_server().await;
        mock_opencode_session_and_message(&mock_opencode).await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Approval Test",
                "description": "Test that planning_review awaits approval"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/execute", task_id))
            .await;

        let execute_response = server
            .post(&format!("/api/tasks/{}/execute", task_id))
            .await;

        execute_response.assert_status_ok();
        let body: Value = execute_response.json();
        
        assert_eq!(body["result"]["type"], "awaiting_approval");
        assert_eq!(body["result"]["phase"], "planning");
    }
}

mod workspace_flow {
    use super::*;

    #[tokio::test]
    async fn test_create_workspace_for_task() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Workspace Test",
                "description": "Test workspace creation"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        let workspace_response = server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        workspace_response.assert_status(axum::http::StatusCode::CREATED);
        let workspace: Value = workspace_response.json();
        
        assert_eq!(workspace["task_id"], task_id);
        assert!(workspace["path"].is_string());
        assert!(workspace["branch_name"].is_string());
        assert!(workspace["status"].is_string());
    }

    #[tokio::test]
    async fn test_list_workspaces_after_creation() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "List Workspace Test",
                "description": "Test listing workspaces"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let list_response = server.get("/api/workspaces").await;
        list_response.assert_status_ok();
        
        let workspaces: Value = list_response.json();
        assert!(workspaces.is_array());
        let workspaces_arr = workspaces.as_array().unwrap();
        assert!(!workspaces_arr.is_empty());
        
        let found = workspaces_arr.iter().any(|ws| ws["task_id"] == task_id);
        assert!(found, "Should find workspace with task_id {}", task_id);
    }

    #[tokio::test]
    async fn test_get_workspace_status() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Status Test",
                "description": "Test workspace status"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let status_response = server
            .get(&format!("/api/workspaces/{}", task_id))
            .await;

        status_response.assert_status_ok();
        let status: Value = status_response.json();
        assert_eq!(status["task_id"], task_id);
    }

    #[tokio::test]
    async fn test_get_workspace_diff_empty() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Diff Test",
                "description": "Test workspace diff"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let diff_response = server
            .get(&format!("/api/workspaces/{}/diff", task_id))
            .await;

        diff_response.assert_status_ok();
        let diff: Value = diff_response.json();
        assert_eq!(diff["task_id"], task_id);
        assert!(diff["diff"].is_string());
    }

    #[tokio::test]
    async fn test_delete_workspace() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let create_response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "Delete Workspace Test",
                "description": "Test workspace deletion"
            }))
            .await;

        let created: Value = create_response.json();
        let task_id = created["id"].as_str().unwrap();

        server
            .post(&format!("/api/tasks/{}/workspace", task_id))
            .await;

        let delete_response = server
            .delete(&format!("/api/workspaces/{}", task_id))
            .await;

        delete_response.assert_status(axum::http::StatusCode::NO_CONTENT);

        let list_response = server.get("/api/workspaces").await;
        let workspaces: Value = list_response.json();
        let workspaces_arr = workspaces.as_array().unwrap();
        let found = workspaces_arr.iter().any(|ws| ws["task_id"] == task_id);
        assert!(!found, "Workspace should be deleted");
    }

    #[tokio::test]
    async fn test_workspace_not_found_for_nonexistent_task() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let fake_id = uuid::Uuid::new_v4();
        let response = server
            .post(&format!("/api/tasks/{}/workspace", fake_id))
            .await;

        response.assert_status_not_found();
    }
}

mod validation {
    use super::*;

    #[tokio::test]
    async fn test_create_task_with_empty_title_fails() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "",
                "description": "Valid description"
            }))
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_create_task_with_whitespace_title_fails() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let response = server
            .post("/api/tasks")
            .json(&json!({
                "title": "   ",
                "description": "Valid description"
            }))
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_transition_nonexistent_task_returns_404() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let fake_id = uuid::Uuid::new_v4();
        let response = server
            .post(&format!("/api/tasks/{}/transition", fake_id))
            .json(&json!({ "status": "planning" }))
            .await;

        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_execute_nonexistent_task_returns_404() {
        let (server, _temp_dir, _mock, _lock) = setup_test_server().await;

        let fake_id = uuid::Uuid::new_v4();
        let response = server
            .post(&format!("/api/tasks/{}/execute", fake_id))
            .await;

        response.assert_status_not_found();
    }
}
