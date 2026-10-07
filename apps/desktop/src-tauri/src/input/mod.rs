//! Platform-abstracted mouse controller.
//!
//! Provides a `MouseController` trait with platform-specific implementations.
//! The networking layer calls these methods without knowing the OS.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

/// Result type for mouse operations.
pub type InputResult<T> = Result<T, InputError>;

/// Mouse input errors.
#[derive(Debug)]
pub enum InputError {
    /// OS-level API call failed.
    PlatformError(String),
    /// Required permission not granted (e.g., macOS Accessibility).
    PermissionDenied(String),
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlatformError(msg) => write!(f, "platform error: {msg}"),
            Self::PermissionDenied(msg) => write!(f, "permission denied: {msg}"),
        }
    }
}

impl std::error::Error for InputError {}

/// Platform-abstracted mouse controller.
///
/// Each platform implements this trait using native APIs.
/// The networking/protocol layer calls these methods exclusively.
pub trait MouseController: Send + Sync {
    /// Move the cursor by a relative offset.
    fn move_relative(&self, dx: i32, dy: i32) -> InputResult<()>;

    /// Press the left mouse button.
    fn left_down(&self) -> InputResult<()>;

    /// Release the left mouse button.
    fn left_up(&self) -> InputResult<()>;

    /// Press the right mouse button.
    fn right_down(&self) -> InputResult<()>;

    /// Release the right mouse button.
    fn right_up(&self) -> InputResult<()>;

    /// Scroll by relative amounts.
    /// Positive dy = scroll up, negative dy = scroll down.
    /// Positive dx = scroll right, negative dx = scroll left.
    fn scroll(&self, dx: i32, dy: i32) -> InputResult<()>;

    /// Release all mouse buttons. Called on disconnect to prevent stuck buttons.
    fn reset(&self) -> InputResult<()>;

    /// Check if the required permissions are granted.
    fn check_permissions(&self) -> InputResult<bool>;
}

/// Create the platform-appropriate mouse controller.
pub fn create_mouse_controller() -> Box<dyn MouseController> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSMouseController::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsMouseController::new())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        compile_error!("Unsupported platform. MovaPad supports macOS and Windows only.");
    }
}
