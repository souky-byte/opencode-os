use crate::error::DbError;
use crate::models::{CreateSessionActivity, SessionActivity, SessionActivityRow};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionActivityRepository {
    pool: SqlitePool,
}

impl SessionActivityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, activity: &CreateSessionActivity) -> Result<i64, DbError> {
        let data_json =
            serde_json::to_string(&activity.data).unwrap_or_else(|_| "null".to_string());
        let created_at = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO session_activities (session_id, activity_type, activity_id, data, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(activity.session_id.to_string())
        .bind(&activity.activity_type)
        .bind(&activity.activity_id)
        .bind(&data_json)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn find_by_session_id(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<SessionActivity>, DbError> {
        let rows: Vec<SessionActivityRow> = sqlx::query_as(
            r#"
            SELECT id, session_id, activity_type, activity_id, data, created_at
            FROM session_activities
            WHERE session_id = ?
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(session_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    pub async fn find_by_session_id_since(
        &self,
        session_id: Uuid,
        since_id: i64,
    ) -> Result<Vec<SessionActivity>, DbError> {
        let rows: Vec<SessionActivityRow> = sqlx::query_as(
            r#"
            SELECT id, session_id, activity_type, activity_id, data, created_at
            FROM session_activities
            WHERE session_id = ? AND id > ?
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(session_id.to_string())
        .bind(since_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    pub async fn count_by_session_id(&self, session_id: Uuid) -> Result<i64, DbError> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_activities WHERE session_id = ?")
                .bind(session_id.to_string())
                .fetch_one(&self.pool)
                .await?;

        Ok(count.0)
    }

    pub async fn delete_by_session_id(&self, session_id: Uuid) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM session_activities WHERE session_id = ?")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_pool, run_migrations, SessionRepository, TaskRepository};
    use opencode_core::{Session, SessionPhase, Task};
    use serde_json::json;

    async fn setup_test_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    async fn create_test_session(pool: &SqlitePool) -> Session {
        let task_repo = TaskRepository::new(pool.clone());
        let task = Task::new("Test Task", "Test Description");
        task_repo.create(&task).await.unwrap();

        let session_repo = SessionRepository::new(pool.clone());
        let session = Session::new(task.id, SessionPhase::Planning);
        session_repo.create(&session).await.unwrap();
        session
    }

    #[tokio::test]
    async fn test_create_activity() {
        let pool = setup_test_db().await;
        let session = create_test_session(&pool).await;
        let repo = SessionActivityRepository::new(pool);

        let activity = CreateSessionActivity::new(
            session.id,
            "tool_call",
            Some("tc-1".to_string()),
            json!({"tool_name": "read_file", "args": {"path": "/test.txt"}}),
        );

        let id = repo.create(&activity).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_find_by_session_id() {
        let pool = setup_test_db().await;
        let session = create_test_session(&pool).await;
        let repo = SessionActivityRepository::new(pool);

        // Create multiple activities
        for i in 0..3 {
            let activity = CreateSessionActivity::new(
                session.id,
                "agent_message",
                Some(format!("msg-{}", i)),
                json!({"content": format!("Message {}", i)}),
            );
            repo.create(&activity).await.unwrap();
        }

        let activities = repo.find_by_session_id(session.id).await.unwrap();
        assert_eq!(activities.len(), 3);

        // Check ordering (oldest first)
        assert!(activities[0].id < activities[1].id);
        assert!(activities[1].id < activities[2].id);
    }

    #[tokio::test]
    async fn test_find_by_session_id_since() {
        let pool = setup_test_db().await;
        let session = create_test_session(&pool).await;
        let repo = SessionActivityRepository::new(pool);

        // Create activities and track IDs
        let mut ids = vec![];
        for i in 0..5 {
            let activity = CreateSessionActivity::new(
                session.id,
                "tool_call",
                Some(format!("tc-{}", i)),
                json!({"tool": format!("tool_{}", i)}),
            );
            ids.push(repo.create(&activity).await.unwrap());
        }

        // Get activities since id[2]
        let activities = repo
            .find_by_session_id_since(session.id, ids[2])
            .await
            .unwrap();
        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].id, ids[3]);
        assert_eq!(activities[1].id, ids[4]);
    }

    #[tokio::test]
    async fn test_count_by_session_id() {
        let pool = setup_test_db().await;
        let session = create_test_session(&pool).await;
        let repo = SessionActivityRepository::new(pool);

        for i in 0..5 {
            let activity = CreateSessionActivity::new(
                session.id,
                "tool_result",
                Some(format!("tr-{}", i)),
                json!({"result": "ok"}),
            );
            repo.create(&activity).await.unwrap();
        }

        let count = repo.count_by_session_id(session.id).await.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_delete_by_session_id() {
        let pool = setup_test_db().await;
        let session = create_test_session(&pool).await;
        let repo = SessionActivityRepository::new(pool);

        for i in 0..3 {
            let activity = CreateSessionActivity::new(
                session.id,
                "finished",
                None,
                json!({"success": true, "index": i}),
            );
            repo.create(&activity).await.unwrap();
        }

        let deleted = repo.delete_by_session_id(session.id).await.unwrap();
        assert_eq!(deleted, 3);

        let remaining = repo.find_by_session_id(session.id).await.unwrap();
        assert!(remaining.is_empty());
    }
}
