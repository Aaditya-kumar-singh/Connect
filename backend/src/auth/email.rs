use crate::domain::errors::AppError;
use tracing::info;

#[axum::async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_otp(&self, recipient: &str, otp: &str, purpose: &str) -> Result<(), AppError>;
}

#[derive(Clone, Default)]
pub struct ConsoleEmailSender;

#[axum::async_trait]
impl EmailSender for ConsoleEmailSender {
    async fn send_otp(&self, recipient: &str, otp: &str, purpose: &str) -> Result<(), AppError> {
        info!(recipient = %recipient, purpose = %purpose, "OTP generated for configured console email provider");
        info!(purpose = %purpose, "OTP delivery delegated to email provider");
        if otp.is_empty() {
            return Err(AppError::ServiceUnavailable(
                "Email delivery unavailable".into(),
            ));
        }
        Ok(())
    }
}
