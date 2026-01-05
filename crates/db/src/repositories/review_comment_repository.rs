use crate::error::DbError;
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReviewComment {
    pub id: String,
    pub task_id: String,
    pub file_path: String,
    pub line_start: i64,
    pub line_end: i64,
    pub side: String,
    pub content: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Clone)]
pub struct ReviewCommentRepository {
    pool: SqlitePool,
}

impl ReviewCommentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all comments for a task
    pub async fn find_by_task_id(&self, task_id: &str) -> Result<Vec<ReviewComment>, DbError> {
        let comments = sqlx::query_as::<_, ReviewComment>(
            r#"
            SELECT id, task_id, file_path, line_start, line_end, side, content, status, created_at
            FROM review_comments
            WHERE task_id = ?
            ORDER BY file_path, line_start
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    /// Get a single comment by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<ReviewComment>, DbError> {
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            SELECT id, task_id, file_path, line_start, line_end, side, content, status, created_at
            FROM review_comments
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(comment)
    }

    /// Create a new comment
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        id: &str,
        task_id: &str,
        file_path: &str,
        line_start: i64,
        line_end: i64,
        side: &str,
        content: &str,
    ) -> Result<ReviewComment, DbError> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO review_comments (id, task_id, file_path, line_start, line_end, side, content, status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?)
            "#,
        )
        .bind(id)
        .bind(task_id)
        .bind(file_path)
        .bind(line_start)
        .bind(line_end)
        .bind(side)
        .bind(content)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ReviewComment {
            id: id.to_string(),
            task_id: task_id.to_string(),
            file_path: file_path.to_string(),
            line_start,
            line_end,
            side: side.to_string(),
            content: content.to_string(),
            status: "pending".to_string(),
            created_at: now,
        })
    }

    /// Update comment content
    pub async fn update_content(&self, id: &str, content: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE review_comments
            SET content = ?
            WHERE id = ?
            "#,
        )
        .bind(content)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update comment status
    pub async fn update_status(&self, id: &str, status: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE review_comments
            SET status = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update status for multiple comments
    pub async fn update_status_bulk(&self, ids: &[String], status: &str) -> Result<(), DbError> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query = format!(
            "UPDATE review_comments SET status = ? WHERE id IN ({})",
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&query).bind(status);
        for id in ids {
            q = q.bind(id);
        }

        q.execute(&self.pool).await?;
        Ok(())
    }

    /// Delete a comment
    pub async fn delete(&self, id: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            DELETE FROM review_comments
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all comments for a task
    pub async fn delete_by_task_id(&self, task_id: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            DELETE FROM review_comments
            WHERE task_id = ?
            "#,
        )
        .bind(task_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get comments by IDs
    pub async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<ReviewComment>, DbError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query = format!(
            r#"
            SELECT id, task_id, file_path, line_start, line_end, side, content, status, created_at
            FROM review_comments
            WHERE id IN ({})
            ORDER BY file_path, line_start
            "#,
            placeholders.join(", ")
        );

        let mut q = sqlx::query_as::<_, ReviewComment>(&query);
        for id in ids {
            q = q.bind(id);
        }

        let comments = q.fetch_all(&self.pool).await?;
        Ok(comments)
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

    async fn create_test_task(pool: &SqlitePool, task_id: &str) {
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, status, created_at, updated_at)
            VALUES (?, 'Test Task', 'Test description', 'todo', ?, ?)
            "#,
        )
        .bind(task_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_find_comment() {
        let pool = setup_test_db().await;
        let repo = ReviewCommentRepository::new(pool.clone());

        create_test_task(&pool, "task-123").await;

        let comment = repo
            .create(
                "comment-1",
                "task-123",
                "src/main.rs",
                10,
                15,
                "new",
                "This needs refactoring",
            )
            .await
            .unwrap();

        assert_eq!(comment.id, "comment-1");
        assert_eq!(comment.task_id, "task-123");
        assert_eq!(comment.line_start, 10);
        assert_eq!(comment.line_end, 15);
        assert_eq!(comment.status, "pending");

        let found = repo.find_by_id("comment-1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "This needs refactoring");
    }

    #[tokio::test]
    async fn test_find_by_task_id() {
        let pool = setup_test_db().await;
        let repo = ReviewCommentRepository::new(pool.clone());

        create_test_task(&pool, "task-1").await;
        create_test_task(&pool, "task-2").await;

        repo.create("c1", "task-1", "src/a.rs", 1, 5, "new", "Comment 1")
            .await
            .unwrap();
        repo.create("c2", "task-1", "src/b.rs", 10, 20, "old", "Comment 2")
            .await
            .unwrap();
        repo.create("c3", "task-2", "src/c.rs", 5, 5, "new", "Other task")
            .await
            .unwrap();

        let comments = repo.find_by_task_id("task-1").await.unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[tokio::test]
    async fn test_update_status() {
        let pool = setup_test_db().await;
        let repo = ReviewCommentRepository::new(pool.clone());

        create_test_task(&pool, "task-1").await;

        repo.create("c1", "task-1", "src/a.rs", 1, 5, "new", "Comment")
            .await
            .unwrap();

        repo.update_status("c1", "sent").await.unwrap();

        let comment = repo.find_by_id("c1").await.unwrap().unwrap();
        assert_eq!(comment.status, "sent");
    }

    #[tokio::test]
    async fn test_delete_comment() {
        let pool = setup_test_db().await;
        let repo = ReviewCommentRepository::new(pool.clone());

        create_test_task(&pool, "task-1").await;

        repo.create("c1", "task-1", "src/a.rs", 1, 5, "new", "Comment")
            .await
            .unwrap();

        repo.delete("c1").await.unwrap();

        let found = repo.find_by_id("c1").await.unwrap();
        assert!(found.is_none());
    }
}
