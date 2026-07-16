use serde::Serialize;

/// Serializable error returned from Tauri commands.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Generic internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }

    /// Usage data could not be retrieved.
    pub fn usage_unavailable(message: impl Into<String>) -> Self {
        Self::new("usage_unavailable", message)
    }

    /// Settings could not be loaded or saved.
    pub fn settings_error(message: impl Into<String>) -> Self {
        Self::new("settings_error", message)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_serializes() {
        let err = AppError::new("test_code", "test message");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("test_code"));
        assert!(json.contains("test message"));
    }

    #[test]
    fn app_error_fields_are_public() {
        let err = AppError::new("code", "message");
        assert_eq!(err.code, "code");
        assert_eq!(err.message, "message");
    }

    #[test]
    fn usage_unavailable_factory() {
        let err = AppError::usage_unavailable("session not found");
        assert_eq!(err.code, "usage_unavailable");
        assert_eq!(err.message, "session not found");
    }

    #[test]
    fn internal_factory() {
        let err = AppError::internal("something broke");
        assert_eq!(err.code, "internal");
        assert_eq!(err.message, "something broke");
    }
}
