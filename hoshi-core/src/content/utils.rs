use std::sync::Arc;
use serde_json::Value;
use crate::state::AppState;
use crate::config::repository::ConfigRepository;

pub async fn show_adult(state: &AppState, user_id: i32) -> bool {
    ConfigRepository::get_config(&state.pool, user_id)
        .await
        .map(|c| c.general.show_adult_content)
        .unwrap_or(false)
}

pub fn filter_home_nsfw(mut view: Value) -> Value {
    if let Some(obj) = view.as_object_mut() {
        for section_key in ["anime", "manga", "novel"] {
            if let Some(section) = obj.get_mut(section_key).and_then(|s| s.as_object_mut()) {
                for list_key in ["trending", "topRated", "seasonal"] {
                    if let Some(arr) = section.get_mut(list_key).and_then(|v| v.as_array_mut()) {
                        arr.retain(|item| {
                            !item.get("content")
                                .and_then(|c| c.get("nsfw"))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                        });
                    }
                }
            }
        }
    }
    view
}

pub fn filter_array_nsfw(value: Value) -> Value {
    if let Value::Array(mut arr) = value {
        arr.retain(|item| {
            !item.get("content")
                .and_then(|c| c.get("nsfw"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        });
        Value::Array(arr)
    } else {
        value
    }
}