//! Vector store using SQLite + sqlite-vec for similarity search

use std::path::Path;
use std::sync::Once;

use rusqlite::{ffi::sqlite3_auto_extension, params, Connection};
use tracing::{debug, info};
use uuid::Uuid;

use crate::domain::{
    chunk::{ChunkType, CodeChunk},
    index_status::{IndexState, IndexStatus},
    search_result::SearchResult,
    wiki_page::{Importance, PageType, SourceCitation, WikiPage, WikiStructure, WikiTree},
    wiki_section::WikiSection,
};
use crate::error::{WikiError, WikiResult};

/// Embedding dimension for text-embedding-3-small
pub const EMBEDDING_DIMENSION: usize = 1536;

static SQLITE_VEC_INIT: Once = Once::new();

fn init_sqlite_vec_extension() {
    SQLITE_VEC_INIT.call_once(|| unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *mut i8,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> i32,
        >(sqlite_vec::sqlite3_vec_init as *const ())));
    });
}

/// Vector store backed by SQLite with sqlite-vec extension
pub struct VectorStore {
    conn: Connection,
}

impl VectorStore {
    /// Create a new VectorStore, initializing the database if needed
    pub fn new(db_path: &Path) -> WikiResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Register sqlite-vec as an auto-extension (must be done before opening connection)
        init_sqlite_vec_extension();

        let conn = Connection::open(db_path)?;

        let vec_version: String = conn.query_row("SELECT vec_version()", [], |row| row.get(0))?;
        debug!("sqlite-vec version: {}", vec_version);

        let store = Self { conn };
        store.init_schema()?;

        info!("VectorStore initialized at {:?}", db_path);
        Ok(store)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> WikiResult<()> {
        self.conn.execute_batch(
            r#"
            -- Code chunks table
            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                branch TEXT NOT NULL,
                file_path TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                content TEXT NOT NULL,
                chunk_type TEXT NOT NULL,
                language TEXT,
                token_count INTEGER NOT NULL,
                chunk_index INTEGER NOT NULL,
                commit_sha TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_chunks_branch ON chunks(branch);
            CREATE INDEX IF NOT EXISTS idx_chunks_file_path ON chunks(file_path);

            -- Chunk embeddings using sqlite-vec virtual table
            CREATE VIRTUAL TABLE IF NOT EXISTS chunk_embeddings USING vec0(
                chunk_id TEXT PRIMARY KEY,
                embedding FLOAT[1536]
            );

            -- Wiki pages table
            CREATE TABLE IF NOT EXISTS wiki_pages (
                id TEXT PRIMARY KEY,
                branch TEXT NOT NULL,
                slug TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                page_type TEXT NOT NULL,
                parent_slug TEXT,
                page_order INTEGER NOT NULL,
                file_paths TEXT NOT NULL,
                has_diagrams INTEGER NOT NULL,
                commit_sha TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(branch, slug)
            );

            CREATE INDEX IF NOT EXISTS idx_wiki_pages_branch ON wiki_pages(branch);
            CREATE INDEX IF NOT EXISTS idx_wiki_pages_parent ON wiki_pages(parent_slug);

            -- Index status table
            CREATE TABLE IF NOT EXISTS index_status (
                branch TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                last_commit_sha TEXT,
                file_count INTEGER NOT NULL DEFAULT 0,
                chunk_count INTEGER NOT NULL DEFAULT 0,
                page_count INTEGER NOT NULL DEFAULT 0,
                last_indexed_at TEXT,
                error_message TEXT,
                progress_percent INTEGER NOT NULL DEFAULT 0,
                current_phase TEXT,
                current_item TEXT
            );

            -- Wiki structure cache
            CREATE TABLE IF NOT EXISTS wiki_structure (
                branch TEXT PRIMARY KEY,
                structure_json TEXT NOT NULL,
                page_count INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Wiki sections table (for hierarchical organization)
            CREATE TABLE IF NOT EXISTS wiki_sections (
                id TEXT PRIMARY KEY,
                branch TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                page_slugs TEXT NOT NULL DEFAULT '[]',
                subsection_ids TEXT NOT NULL DEFAULT '[]',
                order_num INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_wiki_sections_branch ON wiki_sections(branch);
            "#,
        )?;

        self.migrate_index_status_columns()?;
        self.migrate_wiki_pages_columns()?;

        debug!("Database schema initialized");
        Ok(())
    }

    fn migrate_index_status_columns(&self) -> WikiResult<()> {
        let columns_to_add = [
            ("page_count", "INTEGER NOT NULL DEFAULT 0"),
            ("current_phase", "TEXT"),
            ("current_item", "TEXT"),
        ];

        for (column_name, column_def) in columns_to_add {
            let column_exists: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('index_status') WHERE name = ?1",
                params![column_name],
                |row| row.get(0),
            )?;

            if !column_exists {
                let sql = format!(
                    "ALTER TABLE index_status ADD COLUMN {} {}",
                    column_name, column_def
                );
                self.conn.execute(&sql, [])?;
                debug!("Added column {} to index_status table", column_name);
            }
        }

        Ok(())
    }

    fn migrate_wiki_pages_columns(&self) -> WikiResult<()> {
        let columns_to_add = [
            ("importance", "TEXT DEFAULT 'medium'"),
            ("related_pages", "TEXT DEFAULT '[]'"),
            ("section_id", "TEXT"),
            ("source_citations", "TEXT DEFAULT '[]'"),
        ];

        for (column_name, column_def) in columns_to_add {
            let column_exists: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('wiki_pages') WHERE name = ?1",
                params![column_name],
                |row| row.get(0),
            )?;

            if !column_exists {
                let sql = format!(
                    "ALTER TABLE wiki_pages ADD COLUMN {} {}",
                    column_name, column_def
                );
                self.conn.execute(&sql, [])?;
                debug!("Added column {} to wiki_pages table", column_name);
            }
        }

        Ok(())
    }

    /// Insert a code chunk
    pub fn insert_chunk(&self, chunk: &CodeChunk) -> WikiResult<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO chunks 
            (id, branch, file_path, start_line, end_line, content, chunk_type, 
             language, token_count, chunk_index, commit_sha, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                chunk.id.to_string(),
                chunk.branch,
                chunk.file_path,
                chunk.start_line,
                chunk.end_line,
                chunk.content,
                chunk.chunk_type.as_str(),
                chunk.language,
                chunk.token_count,
                chunk.chunk_index,
                chunk.commit_sha,
                chunk.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn insert_embedding(&self, chunk_id: &Uuid, embedding: &[f32]) -> WikiResult<()> {
        if embedding.len() != EMBEDDING_DIMENSION {
            return Err(WikiError::DimensionMismatch {
                expected: EMBEDDING_DIMENSION,
                actual: embedding.len(),
            });
        }

        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

        self.conn.execute(
            "INSERT OR REPLACE INTO chunk_embeddings (chunk_id, embedding) VALUES (?1, ?2)",
            params![chunk_id.to_string(), embedding_bytes],
        )?;
        Ok(())
    }

    pub fn insert_chunks_batch(&self, chunks: &[CodeChunk]) -> WikiResult<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        let mut stmt = self.conn.prepare_cached(
            r#"
            INSERT OR REPLACE INTO chunks 
            (id, branch, file_path, start_line, end_line, content, chunk_type, 
             language, token_count, chunk_index, commit_sha, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        )?;

        for chunk in chunks {
            stmt.execute(params![
                chunk.id.to_string(),
                chunk.branch,
                chunk.file_path,
                chunk.start_line,
                chunk.end_line,
                chunk.content,
                chunk.chunk_type.as_str(),
                chunk.language,
                chunk.token_count,
                chunk.chunk_index,
                chunk.commit_sha,
                chunk.created_at.to_rfc3339(),
            ])?;
        }

        Ok(())
    }

    pub fn insert_embeddings_batch(
        &self,
        chunk_ids: &[Uuid],
        embeddings: &[Vec<f32>],
    ) -> WikiResult<()> {
        if chunk_ids.len() != embeddings.len() {
            return Err(WikiError::IndexingFailed(format!(
                "Chunk IDs count ({}) doesn't match embeddings count ({})",
                chunk_ids.len(),
                embeddings.len()
            )));
        }

        if chunk_ids.is_empty() {
            return Ok(());
        }

        let mut stmt = self.conn.prepare_cached(
            "INSERT OR REPLACE INTO chunk_embeddings (chunk_id, embedding) VALUES (?1, ?2)",
        )?;

        for (chunk_id, embedding) in chunk_ids.iter().zip(embeddings.iter()) {
            if embedding.len() != EMBEDDING_DIMENSION {
                return Err(WikiError::DimensionMismatch {
                    expected: EMBEDDING_DIMENSION,
                    actual: embedding.len(),
                });
            }

            let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

            stmt.execute(params![chunk_id.to_string(), embedding_bytes])?;
        }

        Ok(())
    }

    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> WikiResult<Vec<SearchResult>> {
        self.search_similar_in_branch(query_embedding, limit, None)
    }

    pub fn search_similar_in_branch(
        &self,
        query_embedding: &[f32],
        limit: usize,
        branch: Option<&str>,
    ) -> WikiResult<Vec<SearchResult>> {
        if query_embedding.len() != EMBEDDING_DIMENSION {
            return Err(WikiError::DimensionMismatch {
                expected: EMBEDDING_DIMENSION,
                actual: query_embedding.len(),
            });
        }

        let embedding_bytes: Vec<u8> = query_embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let (sql, use_branch_filter) = if branch.is_some() {
            (
                r#"
                SELECT 
                    c.id, c.file_path, c.start_line, c.end_line, c.content,
                    c.chunk_type, c.language,
                    vec_distance_cosine(e.embedding, ?1) as distance
                FROM chunk_embeddings e
                JOIN chunks c ON c.id = e.chunk_id
                WHERE c.branch = ?3
                ORDER BY distance ASC
                LIMIT ?2
                "#,
                true,
            )
        } else {
            (
                r#"
                SELECT 
                    c.id, c.file_path, c.start_line, c.end_line, c.content,
                    c.chunk_type, c.language,
                    vec_distance_cosine(e.embedding, ?1) as distance
                FROM chunk_embeddings e
                JOIN chunks c ON c.id = e.chunk_id
                ORDER BY distance ASC
                LIMIT ?2
                "#,
                false,
            )
        };

        let mut stmt = self.conn.prepare(sql)?;

        let row_mapper = |row: &rusqlite::Row| {
            let id_str: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let start_line: u32 = row.get(2)?;
            let end_line: u32 = row.get(3)?;
            let content: String = row.get(4)?;
            let chunk_type_str: String = row.get(5)?;
            let language: Option<String> = row.get(6)?;
            let distance: f32 = row.get(7)?;

            let score = 1.0 - distance;

            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let chunk_type = ChunkType::parse(&chunk_type_str).unwrap_or(ChunkType::Code);

            Ok(SearchResult::new(
                id, file_path, start_line, end_line, content, chunk_type, language, score,
            ))
        };

        let results = if use_branch_filter {
            stmt.query_map(
                params![embedding_bytes, limit as i64, branch.unwrap()],
                row_mapper,
            )?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![embedding_bytes, limit as i64], row_mapper)?
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(results)
    }

    pub fn get_index_status(&self, branch: &str) -> WikiResult<Option<IndexStatus>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT branch, state, last_commit_sha, file_count, chunk_count, page_count,
                   last_indexed_at, error_message, progress_percent, current_phase, current_item
            FROM index_status
            WHERE branch = ?1
            "#,
        )?;

        let result = stmt.query_row(params![branch], |row| {
            let state_str: String = row.get(1)?;
            let last_indexed_str: Option<String> = row.get(6)?;

            Ok(IndexStatus {
                branch: row.get(0)?,
                state: IndexState::parse(&state_str).unwrap_or(IndexState::NotIndexed),
                last_commit_sha: row.get(2)?,
                file_count: row.get(3)?,
                chunk_count: row.get(4)?,
                page_count: row.get(5)?,
                last_indexed_at: last_indexed_str
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                error_message: row.get(7)?,
                progress_percent: row.get(8)?,
                current_phase: row.get(9)?,
                current_item: row.get(10)?,
            })
        });

        match result {
            Ok(status) => Ok(Some(status)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_index_status(&self, status: &IndexStatus) -> WikiResult<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO index_status 
            (branch, state, last_commit_sha, file_count, chunk_count, page_count,
             last_indexed_at, error_message, progress_percent, current_phase, current_item)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                status.branch,
                status.state.as_str(),
                status.last_commit_sha,
                status.file_count,
                status.chunk_count,
                status.page_count,
                status.last_indexed_at.map(|dt| dt.to_rfc3339()),
                status.error_message,
                status.progress_percent,
                status.current_phase,
                status.current_item,
            ],
        )?;
        Ok(())
    }

    /// Insert a wiki page
    pub fn insert_wiki_page(&self, page: &WikiPage) -> WikiResult<()> {
        let file_paths_json = serde_json::to_string(&page.file_paths)?;
        let related_pages_json = serde_json::to_string(&page.related_pages)?;
        let source_citations_json = serde_json::to_string(&page.source_citations)?;

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO wiki_pages 
            (id, branch, slug, title, content, page_type, parent_slug, 
             page_order, file_paths, has_diagrams, commit_sha, created_at, updated_at,
             importance, related_pages, section_id, source_citations)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                page.id.to_string(),
                page.branch,
                page.slug,
                page.title,
                page.content,
                page.page_type.as_str(),
                page.parent_slug,
                page.order,
                file_paths_json,
                page.has_diagrams,
                page.commit_sha,
                page.created_at.to_rfc3339(),
                page.updated_at.to_rfc3339(),
                page.importance.as_str(),
                related_pages_json,
                page.section_id,
                source_citations_json,
            ],
        )?;
        Ok(())
    }

    pub fn get_wiki_page(&self, slug: &str) -> WikiResult<Option<WikiPage>> {
        self.get_wiki_page_in_branch(slug, None)
    }

    pub fn get_wiki_page_in_branch(
        &self,
        slug: &str,
        branch: Option<&str>,
    ) -> WikiResult<Option<WikiPage>> {
        let (sql, use_branch) = if branch.is_some() {
            (
                r#"
                SELECT id, branch, slug, title, content, page_type, parent_slug,
                       page_order, file_paths, has_diagrams, commit_sha, created_at, updated_at,
                       importance, related_pages, section_id, source_citations
                FROM wiki_pages
                WHERE slug = ?1 AND branch = ?2
                "#,
                true,
            )
        } else {
            (
                r#"
                SELECT id, branch, slug, title, content, page_type, parent_slug,
                       page_order, file_paths, has_diagrams, commit_sha, created_at, updated_at,
                       importance, related_pages, section_id, source_citations
                FROM wiki_pages
                WHERE slug = ?1
                LIMIT 1
                "#,
                false,
            )
        };

        let mut stmt = self.conn.prepare(sql)?;

        let row_mapper = |row: &rusqlite::Row| {
            let id_str: String = row.get(0)?;
            let page_type_str: String = row.get(5)?;
            let file_paths_json: String = row.get(8)?;
            let created_str: String = row.get(11)?;
            let updated_str: String = row.get(12)?;

            let importance_str: Option<String> = row.get(13)?;
            let related_pages_json: Option<String> = row.get(14)?;
            let section_id: Option<String> = row.get(15)?;
            let source_citations_json: Option<String> = row.get(16)?;

            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let file_paths: Vec<String> = serde_json::from_str(&file_paths_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        11,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        12,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            let importance = importance_str
                .and_then(|s| Importance::parse(&s))
                .unwrap_or_default();

            let related_pages: Vec<String> = related_pages_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            let source_citations: Vec<SourceCitation> = source_citations_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(WikiPage {
                id,
                branch: row.get(1)?,
                slug: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                page_type: PageType::parse(&page_type_str).unwrap_or(PageType::Custom),
                parent_slug: row.get(6)?,
                order: row.get(7)?,
                file_paths,
                has_diagrams: row.get(9)?,
                commit_sha: row.get(10)?,
                created_at,
                updated_at,
                importance,
                related_pages,
                section_id,
                source_citations,
            })
        };

        let result = if use_branch {
            stmt.query_row(params![slug, branch.unwrap()], row_mapper)
        } else {
            stmt.query_row(params![slug], row_mapper)
        };

        match result {
            Ok(page) => Ok(Some(page)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get wiki structure for a branch
    pub fn get_wiki_structure(&self, branch: &str) -> WikiResult<Option<WikiStructure>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT structure_json, page_count, updated_at
            FROM wiki_structure
            WHERE branch = ?1
            "#,
        )?;

        let result = stmt.query_row(params![branch], |row| {
            let json: String = row.get(0)?;
            let page_count: u32 = row.get(1)?;
            let updated_str: String = row.get(2)?;

            let root: WikiTree = serde_json::from_str(&json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(WikiStructure {
                branch: branch.to_string(),
                root,
                page_count,
                updated_at,
                sections: Vec::new(),
                root_section_ids: Vec::new(),
            })
        });

        match result {
            Ok(structure) => Ok(Some(structure)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Save wiki structure
    pub fn save_wiki_structure(&self, structure: &WikiStructure) -> WikiResult<()> {
        let json = serde_json::to_string(&structure.root)?;

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO wiki_structure 
            (branch, structure_json, page_count, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                structure.branch,
                json,
                structure.page_count,
                structure.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Delete all data for a branch (for re-indexing)
    pub fn clear_branch(&self, branch: &str) -> WikiResult<()> {
        self.conn.execute(
            r#"
            DELETE FROM chunk_embeddings 
            WHERE chunk_id IN (SELECT id FROM chunks WHERE branch = ?1)
            "#,
            params![branch],
        )?;

        self.conn
            .execute("DELETE FROM chunks WHERE branch = ?1", params![branch])?;
        self.conn
            .execute("DELETE FROM wiki_pages WHERE branch = ?1", params![branch])?;
        self.conn.execute(
            "DELETE FROM wiki_sections WHERE branch = ?1",
            params![branch],
        )?;
        self.conn.execute(
            "DELETE FROM wiki_structure WHERE branch = ?1",
            params![branch],
        )?;
        self.conn.execute(
            "DELETE FROM index_status WHERE branch = ?1",
            params![branch],
        )?;

        debug!("Cleared all data for branch: {}", branch);
        Ok(())
    }

    pub fn insert_wiki_section(&self, section: &WikiSection) -> WikiResult<()> {
        let page_slugs_json = serde_json::to_string(&section.page_slugs)?;
        let subsection_ids_json = serde_json::to_string(&section.subsection_ids)?;

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO wiki_sections 
            (id, branch, title, description, page_slugs, subsection_ids, order_num, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                section.id,
                section.branch,
                section.title,
                section.description,
                page_slugs_json,
                subsection_ids_json,
                section.order,
                section.created_at.to_rfc3339(),
                section.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_wiki_sections(&self, branch: &str) -> WikiResult<Vec<WikiSection>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, branch, title, description, page_slugs, subsection_ids, order_num, created_at, updated_at
            FROM wiki_sections
            WHERE branch = ?1
            ORDER BY order_num
            "#,
        )?;

        let sections = stmt
            .query_map(params![branch], |row| {
                let page_slugs_json: String = row.get(4)?;
                let subsection_ids_json: String = row.get(5)?;
                let created_str: String = row.get(7)?;
                let updated_str: String = row.get(8)?;

                let page_slugs: Vec<String> =
                    serde_json::from_str(&page_slugs_json).unwrap_or_default();
                let subsection_ids: Vec<String> =
                    serde_json::from_str(&subsection_ids_json).unwrap_or_default();

                let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                Ok(WikiSection {
                    id: row.get(0)?,
                    branch: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    page_slugs,
                    subsection_ids,
                    order: row.get(6)?,
                    created_at,
                    updated_at,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sections)
    }

    pub fn get_wiki_section(
        &self,
        section_id: &str,
        branch: &str,
    ) -> WikiResult<Option<WikiSection>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, branch, title, description, page_slugs, subsection_ids, order_num, created_at, updated_at
            FROM wiki_sections
            WHERE id = ?1 AND branch = ?2
            "#,
        )?;

        let result = stmt.query_row(params![section_id, branch], |row| {
            let page_slugs_json: String = row.get(4)?;
            let subsection_ids_json: String = row.get(5)?;
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;

            let page_slugs: Vec<String> =
                serde_json::from_str(&page_slugs_json).unwrap_or_default();
            let subsection_ids: Vec<String> =
                serde_json::from_str(&subsection_ids_json).unwrap_or_default();

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(WikiSection {
                id: row.get(0)?,
                branch: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                page_slugs,
                subsection_ids,
                order: row.get(6)?,
                created_at,
                updated_at,
            })
        });

        match result {
            Ok(section) => Ok(Some(section)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get chunk count for a branch
    pub fn get_chunk_count(&self, branch: &str) -> WikiResult<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM chunks WHERE branch = ?1",
            params![branch],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get page count for a branch
    pub fn get_page_count(&self, branch: &str) -> WikiResult<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM wiki_pages WHERE branch = ?1",
            params![branch],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_store() -> (VectorStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = VectorStore::new(&db_path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_vector_store_creation() {
        let (store, _dir) = create_test_store();
        assert!(store.get_chunk_count("main").unwrap() == 0);
    }

    #[test]
    fn test_chunk_insert_and_count() {
        let (store, _dir) = create_test_store();

        let chunk = CodeChunk::new(
            "main".to_string(),
            "src/lib.rs".to_string(),
            1,
            10,
            "fn test() {}".to_string(),
            ChunkType::Function,
            Some("rust".to_string()),
            5,
            0,
            "abc123".to_string(),
        );

        store.insert_chunk(&chunk).unwrap();
        assert_eq!(store.get_chunk_count("main").unwrap(), 1);
    }

    #[test]
    fn test_index_status() {
        let (store, _dir) = create_test_store();

        // Initially no status
        assert!(store.get_index_status("main").unwrap().is_none());

        // Create and save status
        let status = IndexStatus {
            branch: "main".to_string(),
            state: IndexState::Indexing,
            last_commit_sha: Some("abc123".to_string()),
            file_count: 10,
            chunk_count: 50,
            last_indexed_at: Some(chrono::Utc::now()),
            error_message: None,
            progress_percent: 50,
            page_count: 0,
            current_phase: None,
            current_item: None,
        };

        store.update_index_status(&status).unwrap();

        // Retrieve and verify
        let retrieved = store.get_index_status("main").unwrap().unwrap();
        assert_eq!(retrieved.branch, "main");
        assert_eq!(retrieved.state, IndexState::Indexing);
        assert_eq!(retrieved.file_count, 10);
    }

    #[test]
    fn test_clear_branch() {
        let (store, _dir) = create_test_store();

        // Insert some data
        let chunk = CodeChunk::new(
            "main".to_string(),
            "src/lib.rs".to_string(),
            1,
            10,
            "fn test() {}".to_string(),
            ChunkType::Function,
            Some("rust".to_string()),
            5,
            0,
            "abc123".to_string(),
        );
        store.insert_chunk(&chunk).unwrap();

        let status = IndexStatus::new("main".to_string());
        store.update_index_status(&status).unwrap();

        // Clear
        store.clear_branch("main").unwrap();

        // Verify cleared
        assert_eq!(store.get_chunk_count("main").unwrap(), 0);
        assert!(store.get_index_status("main").unwrap().is_none());
    }
}
