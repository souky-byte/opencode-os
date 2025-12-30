use crate::error::DbError;
use crate::models::TaskRow;
use chrono::Utc;
use opencode_core::{Task, UpdateTaskRequest};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TaskRepository {
    pool: SqlitePool,
}

impl TaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task: &Task) -> Result<Task, DbError> {
        let row = TaskRow::from(task);

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, status, roadmap_item_id, workspace_path, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&row.id)
        .bind(&row.title)
        .bind(&row.description)
        .bind(&row.status)
        .bind(&row.roadmap_item_id)
        .bind(&row.workspace_path)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(task.clone())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, DbError> {
        let row: Option<TaskRow> = sqlx::query_as(
            r#"
            SELECT id, title, description, status, roadmap_item_id, workspace_path, created_at, updated_at
            FROM tasks
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_domain()))
    }

    pub async fn find_all(&self) -> Result<Vec<Task>, DbError> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            r#"
            SELECT id, title, description, status, roadmap_item_id, workspace_path, created_at, updated_at
            FROM tasks
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    pub async fn update(
        &self,
        id: Uuid,
        update: &UpdateTaskRequest,
    ) -> Result<Option<Task>, DbError> {
        let existing = self.find_by_id(id).await?;
        let Some(mut task) = existing else {
            return Ok(None);
        };

        if let Some(title) = &update.title {
            task.title = title.clone();
        }
        if let Some(description) = &update.description {
            task.description = description.clone();
        }
        if let Some(status) = &update.status {
            task.status = *status;
        }
        if let Some(workspace_path) = &update.workspace_path {
            task.workspace_path = Some(workspace_path.clone());
        }

        task.updated_at = Utc::now();
        let row = TaskRow::from(&task);

        sqlx::query(
            r#"
            UPDATE tasks
            SET title = ?, description = ?, status = ?, workspace_path = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&row.title)
        .bind(&row.description)
        .bind(&row.status)
        .bind(&row.workspace_path)
        .bind(row.updated_at)
        .bind(&row.id)
        .execute(&self.pool)
        .await?;

        Ok(Some(task))
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_pool, run_migrations};
    use opencode_core::TaskStatus;

    async fn setup_test_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_find_task() {
        let pool = setup_test_db().await;
        let repo = TaskRepository::new(pool);

        let task = Task::new("Test Task", "Test Description");
        let created = repo.create(&task).await.unwrap();

        assert_eq!(created.title, "Test Task");

        let found = repo.find_by_id(task.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Task");
    }

    #[tokio::test]
    async fn test_find_all_tasks() {
        let pool = setup_test_db().await;
        let repo = TaskRepository::new(pool);

        repo.create(&Task::new("Task 1", "Desc 1")).await.unwrap();
        repo.create(&Task::new("Task 2", "Desc 2")).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update_task() {
        let pool = setup_test_db().await;
        let repo = TaskRepository::new(pool);

        let task = Task::new("Original", "Description");
        repo.create(&task).await.unwrap();

        let update = UpdateTaskRequest {
            title: Some("Updated".to_string()),
            status: Some(TaskStatus::InProgress),
            ..Default::default()
        };

        let updated = repo.update(task.id, &update).await.unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let pool = setup_test_db().await;
        let repo = TaskRepository::new(pool);

        let task = Task::new("To Delete", "Description");
        repo.create(&task).await.unwrap();

        let deleted = repo.delete(task.id).await.unwrap();
        assert!(deleted);

        let found = repo.find_by_id(task.id).await.unwrap();
        assert!(found.is_none());
    }
}
