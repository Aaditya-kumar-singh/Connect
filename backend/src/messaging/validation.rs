use crate::domain::errors::AppError;
use base64::Engine;
use uuid::Uuid;

pub fn validate_text(content: &str) -> Result<(), AppError> {
    let count = content.chars().count();
    if !(1..=4096).contains(&count) {
        return Err(AppError::Validation(
            "Text content must be 1-4096 characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_content_type(content_type: &str) -> Result<(), AppError> {
    if content_type != "text" {
        return Err(AppError::Validation(
            "Only text messages are supported in Phase 6".into(),
        ));
    }
    Ok(())
}

pub fn validate_emoji(emoji: &str) -> Result<(), AppError> {
    let count = emoji.chars().count();
    if !(1..=20).contains(&count) {
        return Err(AppError::Validation("Emoji must be 1-20 characters".into()));
    }
    Ok(())
}

pub fn validate_reaction_action(action: &str) -> Result<(), AppError> {
    if matches!(action, "add" | "remove") {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Reaction action must be add or remove".into(),
        ))
    }
}

pub fn validate_message_ids(message_ids: &[Uuid]) -> Result<(), AppError> {
    if message_ids.is_empty() {
        return Err(AppError::Validation(
            "message_ids must contain at least one message".into(),
        ));
    }
    Ok(())
}

pub fn encode_cursor(timestamp: &str, id: Uuid) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!("{timestamp}|{id}"))
}

pub fn decode_cursor(cursor: Option<&str>) -> Result<Option<(String, Uuid)>, AppError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::Validation("Invalid cursor".into()))?;
    let value =
        String::from_utf8(bytes).map_err(|_| AppError::Validation("Invalid cursor".into()))?;
    let (timestamp, id) = value
        .split_once('|')
        .ok_or_else(|| AppError::Validation("Invalid cursor".into()))?;
    let id = Uuid::parse_str(id).map_err(|_| AppError::Validation("Invalid cursor".into()))?;
    Ok(Some((timestamp.to_owned(), id)))
}

pub fn validate_limit(limit: Option<u8>) -> Result<i64, AppError> {
    let limit = limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(AppError::Validation(
            "Limit must be between 1 and 100".into(),
        ));
    }
    Ok(limit as i64)
}
