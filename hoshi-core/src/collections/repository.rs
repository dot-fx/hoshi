use crate::error::CoreResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub is_private: bool,
    pub cover_id: Option<String>,
    pub created_at: i64,
}

pub struct CollectionRepo;

impl CollectionRepo {
    pub fn create(conn: &Connection, col: &Collection) -> CoreResult<()> {
        conn.execute(
            "INSERT INTO collections
            (id, user_id, name, description, is_private, cover_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                col.id,
                col.user_id,
                col.name,
                col.description,
                col.is_private as i32,
                col.cover_id,
                col.created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_by_user(conn: &Connection, user_id: i32) -> CoreResult<Vec<Collection>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM collections WHERE user_id = ?"
        )?;

        let rows = stmt.query_map([user_id], |row| {
            Ok(Collection {
                id: row.get("id")?,
                user_id: row.get("user_id")?,
                name: row.get("name")?,
                description: row.get("description")?,
                is_private: row.get::<_, i32>("is_private")? == 1,
                cover_id: row.get("cover_id")?,
                created_at: row.get("created_at")?,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn update(conn: &Connection, col: &Collection) -> CoreResult<()> {
        conn.execute(
            "UPDATE collections
             SET name = ?, description = ?, is_private = ?, cover_id = ?
             WHERE id = ? AND user_id = ?",
            params![
                col.name,
                col.description,
                col.is_private as i32,
                col.cover_id,
                col.id,
                col.user_id
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str, user_id: i32) -> CoreResult<()> {
        conn.execute(
            "DELETE FROM collections WHERE id = ? AND user_id = ?",
            params![id, user_id],
        )?;
        Ok(())
    }

    pub fn add_image(
        conn: &mut Connection,
        collection_id: &str,
        image_id: &str,
        added_at: i64,
    ) -> CoreResult<()> {
        let tx = conn.transaction()?;

        let position: i64 = tx.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1
             FROM collection_images
             WHERE collection_id = ?",
            [collection_id],
            |row| row.get(0),
        )?;

        tx.execute(
            "INSERT INTO collection_images
            (collection_id, image_id, added_at, position)
            VALUES (?, ?, ?, ?)",
            params![collection_id, image_id, added_at, position],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn remove_image(
        conn: &Connection,
        collection_id: &str,
        image_id: &str,
    ) -> CoreResult<()> {
        conn.execute(
            "DELETE FROM collection_images
             WHERE collection_id = ? AND image_id = ?",
            params![collection_id, image_id],
        )?;
        Ok(())
    }

    pub fn reorder(
        conn: &mut Connection,
        collection_id: &str,
        ordered_ids: Vec<String>,
    ) -> CoreResult<()> {
        let tx = conn.transaction()?;

        for (index, image_id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE collection_images
                 SET position = ?
                 WHERE collection_id = ? AND image_id = ?",
                params![index as i64, collection_id, image_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }
}