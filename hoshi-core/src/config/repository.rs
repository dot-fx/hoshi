use crate::error::CoreResult;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

pub struct ConfigRepo;

impl ConfigRepo {
    /// Obtiene la config de un usuario. Si no existe, devuelve `{}`.
    pub fn get_config(conn: &Connection, user_id: i32) -> CoreResult<Value> {
        let result = conn
            .query_row(
                "SELECT config FROM UserConfig WHERE user_id = ?",
                [user_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        match result {
            Some(raw) => {
                let parsed = serde_json::from_str(&raw)
                    .unwrap_or_else(|_| Value::Object(Default::default()));
                Ok(parsed)
            }
            None => Ok(Value::Object(Default::default())),
        }
    }

    /// Hace upsert de la config completa (reemplaza todo el blob).
    pub fn set_config(conn: &Connection, user_id: i32, config: &Value) -> CoreResult<()> {
        let raw = serde_json::to_string(config)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO UserConfig (user_id, config, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
                config = excluded.config,
                updated_at = excluded.updated_at",
            params![user_id, raw, now],
        )?;

        Ok(())
    }

    /// Merge parcial: solo sobreescribe los campos del patch, mantiene el resto.
    pub fn patch_config(conn: &Connection, user_id: i32, patch: &Value) -> CoreResult<Value> {
        let mut current = Self::get_config(conn, user_id)?;

        merge_json(&mut current, patch);
        Self::set_config(conn, user_id, &current)?;

        Ok(current)
    }
}

/// Merge recursivo: los campos del `patch` sobreescriben los de `base`.
/// Si ambos son objetos, mergea recursivamente en vez de reemplazar.
fn merge_json(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (key, patch_val) in patch_map {
                let base_val = base_map.entry(key).or_insert(Value::Null);
                merge_json(base_val, patch_val);
            }
        }
        (base, patch) => {
            *base = patch.clone();
        }
    }
}