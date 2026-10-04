use crate::domain::errors::AppError;

pub fn normalize_email(email: &str) -> Result<String, AppError> {
    let normalized = email.trim().to_ascii_lowercase();
    if normalized.len() > 255
        || !normalized.contains('@')
        || normalized.starts_with('@')
        || normalized.ends_with('@')
    {
        return Err(AppError::Validation("Invalid email address".into()));
    }
    Ok(normalized)
}

pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 || password.len() > 128 {
        return Err(AppError::Validation(
            "Password must be 8-128 characters".into(),
        ));
    }
    let upper = password.chars().any(|c| c.is_ascii_uppercase());
    let lower = password.chars().any(|c| c.is_ascii_lowercase());
    let digit = password.chars().any(|c| c.is_ascii_digit());
    let special = password.chars().any(|c| !c.is_ascii_alphanumeric());
    if !(upper && lower && digit && special) {
        return Err(AppError::Validation(
            "Password must contain upper, lower, digit, and special characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_display_name(name: &str) -> Result<(), AppError> {
    let len = name.trim().chars().count();
    if !(2..=50).contains(&len) {
        return Err(AppError::Validation(
            "Display name must be 2-50 characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_otp(otp: &str) -> Result<(), AppError> {
    if otp.len() != 6 || !otp.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AppError::Validation("OTP must be 6 digits".into()));
    }
    Ok(())
}

pub fn validate_device_type(device_type: &str) -> Result<(), AppError> {
    if matches!(device_type, "WEB" | "ANDROID") {
        Ok(())
    } else {
        Err(AppError::Validation(
            "device_type must be WEB or ANDROID".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_email() {
        assert_eq!(
            normalize_email("  User@Example.COM ").unwrap(),
            "user@example.com"
        );
    }

    #[test]
    fn rejects_weak_passwords() {
        assert!(validate_password("password").is_err());
        assert!(validate_password("Password1!").is_ok());
    }

    #[test]
    fn validates_otp_and_device_type() {
        assert!(validate_otp("123456").is_ok());
        assert!(validate_otp("12345").is_err());
        assert!(validate_device_type("WEB").is_ok());
        assert!(validate_device_type("IOS").is_err());
    }
}
