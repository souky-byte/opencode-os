use crate::error::DbError;
use crate::models::SessionRow;
use opencode_core::{Session, SessionStatus};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, session: &Session) -> Result<Session, DbError> {
        let row = SessionRow::from(session);

        sqlx::query(
            r#"
            INSERT INTO sessions (id, task_id, opencode_session_id, phase, status, started_at, completed_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&row.id)
        .bind(&row.task_id)
        .bind(&row.opencode_session_id)
        .bind(&row.phase)
        .bind(&row.status)
        .bind(row.started_at)
        .bind(row.completed_at)
        .bind(row.created_at)
        .execute(&self.pool)
        .await?;

        Ok(session.clone())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, DbError> {
        let row: Option<SessionRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, opencode_session_id, phase, status, started_at, completed_at, created_at
            FROM sessions
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_domain()))
    }

    pub async fn find_by_task_id(&self, task_id: Uuid) -> Result<Vec<Session>, DbError> {
        let rows: Vec<SessionRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, opencode_session_id, phase, status, started_at, completed_at, created_at
            FROM sessions
            WHERE task_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    pub async fn find_by_opencode_session_id(
        &self,
        opencode_session_id: &str,
    ) -> Result<Option<Session>, DbError> {
        let row: Option<SessionRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, opencode_session_id, phase, status, started_at, completed_at, created_at
            FROM sessions
            WHERE opencode_session_id = ?
            "#,
        )
        .bind(opencode_session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_domain()))
    }

    pub async fn find_all(&self) -> Result<Vec<Session>, DbError> {
        let rows: Vec<SessionRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, opencode_session_id, phase, status, started_at, completed_at, created_at
            FROM sessions
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    pub async fn find_active(&self) -> Result<Vec<Session>, DbError> {
        let rows: Vec<SessionRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, opencode_session_id, phase, status, started_at, completed_at, created_at
            FROM sessions
            WHERE status IN ('pending', 'running')
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    pub async fn update(&self, session: &Session) -> Result<Session, DbError> {
        let row = SessionRow::from(session);

        sqlx::query(
            r#"
            UPDATE sessions
            SET opencode_session_id = ?, phase = ?, status = ?, started_at = ?, completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&row.opencode_session_id)
        .bind(&row.phase)
        .bind(&row.status)
        .bind(row.started_at)
        .bind(row.completed_at)
        .bind(&row.id)
        .execute(&self.pool)
        .await?;

        Ok(session.clone())
    }

    pub async fn update_status(&self, id: Uuid, status: SessionStatus) -> Result<bool, DbError> {
        let result = sqlx::query("UPDATE sessions SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_pool, run_migrations, TaskRepository};
    use opencode_core::{SessionPhase, Task};

    async fn setup_test_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    async fn create_test_task(pool: &SqlitePool) -> Task {
        let task_repo = TaskRepository::new(pool.clone());
        let task = Task::new("Test Task", "Test Description");
        task_repo.create(&task).await.unwrap();
        task
    }

    #[tokio::test]
    async fn test_create_and_find_session() {
        let pool = setup_test_db().await;
        let task = create_test_task(&pool).await;
        let repo = SessionRepository::new(pool);

        let session = Session::new(task.id, SessionPhase::Planning);
        let created = repo.create(&session).await.unwrap();

        assert_eq!(created.task_id, task.id);
        assert_eq!(created.phase, SessionPhase::Planning);

        let found = repo.find_by_id(session.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().phase, SessionPhase::Planning);
    }

    #[tokio::test]
    async fn test_find_by_task_id() {
        let pool = setup_test_db().await;
        let task = create_test_task(&pool).await;
        let repo = SessionRepository::new(pool);

        repo.create(&Session::new(task.id, SessionPhase::Planning))
            .await
            .unwrap();
        repo.create(&Session::new(task.id, SessionPhase::Implementation))
            .await
            .unwrap();

        let sessions = repo.find_by_task_id(task.id).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_update_session() {
        let pool = setup_test_db().await;
        let task = create_test_task(&pool).await;
        let repo = SessionRepository::new(pool);

        let mut session = Session::new(task.id, SessionPhase::Planning);
        repo.create(&session).await.unwrap();

        session.start("opencode-123".to_string());
        repo.update(&session).await.unwrap();

        let found = repo.find_by_id(session.id).await.unwrap().unwrap();
        assert_eq!(found.status, SessionStatus::Running);
        assert_eq!(found.opencode_session_id, Some("opencode-123".to_string()));
    }

    #[tokio::test]
    async fn test_find_active_sessions() {
        let pool = setup_test_db().await;
        let task = create_test_task(&pool).await;
        let repo = SessionRepository::new(pool);

        let mut running = Session::new(task.id, SessionPhase::Planning);
        running.start("opencode-1".to_string());
        repo.create(&running).await.unwrap();

        let mut completed = Session::new(task.id, SessionPhase::Implementation);
        completed.start("opencode-2".to_string());
        completed.complete();
        repo.create(&completed).await.unwrap();

        let active = repo.find_active().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].status, SessionStatus::Running);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let pool = setup_test_db().await;
        let task = create_test_task(&pool).await;
        let repo = SessionRepository::new(pool);

        let session = Session::new(task.id, SessionPhase::Planning);
        repo.create(&session).await.unwrap();

        let deleted = repo.delete(session.id).await.unwrap();
        assert!(deleted);

        let found = repo.find_by_id(session.id).await.unwrap();
        assert!(found.is_none());
    }
}
