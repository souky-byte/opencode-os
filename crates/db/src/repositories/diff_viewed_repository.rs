use crate::error::DbError;
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DiffViewedRepository {
    pool: SqlitePool,
}

impl DiffViewedRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all viewed file paths for a task
    pub async fn get_viewed_files(&self, task_id: &str) -> Result<Vec<String>, DbError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT file_path
            FROM diff_viewed_files
            WHERE task_id = ?
            ORDER BY viewed_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(path,)| path).collect())
    }

    /// Mark a file as viewed
    pub async fn mark_viewed(&self, task_id: &str, file_path: &str) -> Result<(), DbError> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO diff_viewed_files (task_id, file_path, viewed_at)
            VALUES (?, ?, ?)
            ON CONFLICT(task_id, file_path) DO UPDATE SET viewed_at = excluded.viewed_at
            "#,
        )
        .bind(task_id)
        .bind(file_path)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Unmark a file as viewed
    pub async fn unmark_viewed(&self, task_id: &str, file_path: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            DELETE FROM diff_viewed_files
            WHERE task_id = ? AND file_path = ?
            "#,
        )
        .bind(task_id)
        .bind(file_path)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clear all viewed files for a task
    pub async fn clear_viewed_files(&self, task_id: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            DELETE FROM diff_viewed_files
            WHERE task_id = ?
            "#,
        )
        .bind(task_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_pool, run_migrations};

    async fn setup_test_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_mark_and_get_viewed_files() {
        let pool = setup_test_db().await;
        let repo = DiffViewedRepository::new(pool);

        let task_id = "test-task-123";

        // Initially no viewed files
        let viewed = repo.get_viewed_files(task_id).await.unwrap();
        assert!(viewed.is_empty());

        // Mark some files as viewed
        repo.mark_viewed(task_id, "src/main.rs").await.unwrap();
        repo.mark_viewed(task_id, "src/lib.rs").await.unwrap();

        let viewed = repo.get_viewed_files(task_id).await.unwrap();
        assert_eq!(viewed.len(), 2);
        assert!(viewed.contains(&"src/main.rs".to_string()));
        assert!(viewed.contains(&"src/lib.rs".to_string()));
    }

    #[tokio::test]
    async fn test_unmark_viewed() {
        let pool = setup_test_db().await;
        let repo = DiffViewedRepository::new(pool);

        let task_id = "test-task-456";

        repo.mark_viewed(task_id, "src/main.rs").await.unwrap();
        repo.mark_viewed(task_id, "src/lib.rs").await.unwrap();

        repo.unmark_viewed(task_id, "src/main.rs").await.unwrap();

        let viewed = repo.get_viewed_files(task_id).await.unwrap();
        assert_eq!(viewed.len(), 1);
        assert!(viewed.contains(&"src/lib.rs".to_string()));
    }

    #[tokio::test]
    async fn test_clear_viewed_files() {
        let pool = setup_test_db().await;
        let repo = DiffViewedRepository::new(pool);

        let task_id = "test-task-789";

        repo.mark_viewed(task_id, "src/main.rs").await.unwrap();
        repo.mark_viewed(task_id, "src/lib.rs").await.unwrap();

        repo.clear_viewed_files(task_id).await.unwrap();

        let viewed = repo.get_viewed_files(task_id).await.unwrap();
        assert!(viewed.is_empty());
    }

    #[tokio::test]
    async fn test_mark_viewed_idempotent() {
        let pool = setup_test_db().await;
        let repo = DiffViewedRepository::new(pool);

        let task_id = "test-task-idempotent";

        // Mark same file twice should not fail
        repo.mark_viewed(task_id, "src/main.rs").await.unwrap();
        repo.mark_viewed(task_id, "src/main.rs").await.unwrap();

        let viewed = repo.get_viewed_files(task_id).await.unwrap();
        assert_eq!(viewed.len(), 1);
    }
}
