use sqlx::SqlitePool;
use tracing::{debug, instrument};

use crate::content::models::{Relation, RelationType};
use crate::error::CoreResult;

pub struct RelationRepository;

impl RelationRepository {
    #[instrument(skip(pool))]
    pub async fn get_by_source(pool: &SqlitePool, source_cid: &str) -> CoreResult<Vec<Relation>> {
        debug!(cid = %source_cid, "Fetching content relations");

        let rows: Vec<(i64, String, String, String, String, i64)> = sqlx::query_as(
            "SELECT id, source_cid, target_cid, relation_type, source_name, created_at \
             FROM content_relations WHERE source_cid = ?",
        )
            .bind(source_cid)
            .fetch_all(pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|(id, source_cid, target_cid, type_raw, source_name, created_at)| {
                let relation_type = serde_json::from_str(&format!("\"{}\"", type_raw))
                    .unwrap_or(RelationType::Alternative);
                Relation {
                    id: Some(id),
                    source_cid,
                    target_cid,
                    relation_type,
                    source_name,
                    created_at,
                }
            })
            .collect())
    }

    #[instrument(skip(pool, relation))]
    pub async fn upsert(pool: &SqlitePool, relation: &Relation) -> CoreResult<()> {
        debug!(
            source = %relation.source_cid,
            target = %relation.target_cid,
            rel = relation.relation_type.as_str(),
            "Upserting content relation"
        );

        sqlx::query(
            r#"
            INSERT INTO content_relations (source_cid, target_cid, relation_type, source_name, created_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(source_cid, target_cid, relation_type, source_name) DO NOTHING
            "#,
        )
            .bind(&relation.source_cid)
            .bind(&relation.target_cid)
            .bind(relation.relation_type.as_str())
            .bind(&relation.source_name)
            .bind(relation.created_at)
            .execute(pool)
            .await?;

        Ok(())
    }
}