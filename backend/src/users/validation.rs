use crate::domain::errors::AppError;

pub fn validate_profile(input: &super::models::UpdateProfileRequest) -> Result<(), AppError> {
    if let Some(name) = &input.display_name {
        let len = name.trim().chars().count();
        if !(2..=50).contains(&len) {
            return Err(AppError::Validation(
                "Display name must be 2-50 characters".into(),
            ));
        }
    }
    if let Some(bio) = &input.bio {
        if bio.chars().count() > 200 {
            return Err(AppError::Validation(
                "Bio must be at most 200 characters".into(),
            ));
        }
    }
    if let Some(phone) = &input.phone_number {
        if phone.chars().count() > 20 {
            return Err(AppError::Validation(
                "Phone number must be at most 20 characters".into(),
            ));
        }
    }
    Ok(())
}

pub fn validate_device_update(input: &super::models::UpdateDeviceRequest) -> Result<(), AppError> {
    if let Some(provider) = &input.push_provider {
        if !matches!(provider.as_str(), "FCM" | "WEB_PUSH") {
            return Err(AppError::Validation(
                "push_provider must be FCM or WEB_PUSH".into(),
            ));
        }
    }
    if let Some(token) = &input.push_token {
        if token.len() > 4096 {
            return Err(AppError::Validation(
                "push_token must be at most 4096 characters".into(),
            ));
        }
        if input.push_provider.as_deref() == Some("WEB_PUSH") {
            let subscription: serde_json::Value = serde_json::from_str(token).map_err(|_| {
                AppError::Validation("WEB_PUSH push_token must be valid subscription JSON".into())
            })?;
            let endpoint = subscription.get("endpoint").and_then(|v| v.as_str());
            let keys = subscription.get("keys").and_then(|v| v.as_object());
            let p256dh = keys.and_then(|v| v.get("p256dh")).and_then(|v| v.as_str());
            let auth = keys.and_then(|v| v.get("auth")).and_then(|v| v.as_str());
            if endpoint.is_none() || p256dh.is_none() || auth.is_none() {
                return Err(AppError::Validation(
                    "WEB_PUSH subscription must contain endpoint, keys.p256dh and keys.auth".into(),
                ));
            }
        }
    }
    Ok(())
}
