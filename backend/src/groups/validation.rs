use crate::domain::errors::AppError;

pub fn validate_name(name: &str) -> Result<(), AppError> {
    let len = name.trim().chars().count();
    if !(3..=100).contains(&len) {
        return Err(AppError::Validation(
            "Group name must be 3-100 characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_description(description: Option<&str>) -> Result<(), AppError> {
    if let Some(value) = description {
        if value.chars().count() > 500 {
            return Err(AppError::Validation(
                "Group description must be at most 500 characters".into(),
            ));
        }
    }
    Ok(())
}

pub fn validate_members(member_count: usize) -> Result<(), AppError> {
    if !(2..=256).contains(&member_count) {
        return Err(AppError::Validation(
            "A group must contain 2-256 members".into(),
        ));
    }
    Ok(())
}

pub fn validate_role(role: &str) -> Result<(), AppError> {
    if matches!(role, "ADMIN" | "MEMBER") {
        Ok(())
    } else {
        Err(AppError::Validation("Role must be ADMIN or MEMBER".into()))
    }
}

pub fn validate_avatar_url(value: Option<&str>) -> Result<(), AppError> {
    if let Some(value) = value {
        if value.len() > 500 {
            return Err(AppError::Validation(
                "Avatar URL must be at most 500 characters".into(),
            ));
        }
    }
    Ok(())
}
