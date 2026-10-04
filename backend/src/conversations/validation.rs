use crate::domain::errors::AppError;
use base64::Engine;

pub fn validate_limit(limit: Option<u8>) -> Result<i64, AppError> {
    let value = limit.unwrap_or(20);
    if !(1..=50).contains(&value) {
        return Err(AppError::Validation(
            "limit must be between 1 and 50".into(),
        ));
    }
    Ok(i64::from(value))
}

pub fn decode_cursor(cursor: Option<&str>) -> Result<Option<(String, uuid::Uuid)>, AppError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::Validation("Invalid cursor".into()))?;
    let value =
        String::from_utf8(decoded).map_err(|_| AppError::Validation("Invalid cursor".into()))?;
    let (timestamp, id) = value
        .split_once('|')
        .ok_or_else(|| AppError::Validation("Invalid cursor".into()))?;
    uuid::Uuid::parse_str(id)
        .map_err(|_| AppError::Validation("Invalid cursor".into()))
        .map(|id| Some((timestamp.to_owned(), id)))
}

pub fn encode_cursor(timestamp: &str, id: uuid::Uuid) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!("{timestamp}|{id}"))
}
