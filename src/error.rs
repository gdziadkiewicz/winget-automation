use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Winget(#[from] WingetError),

    #[cfg(windows)]
    #[error("failed to create or update the tray icon: {0}")]
    Tray(#[from] tray_item::TIError),

    #[cfg(windows)]
    #[error("failed to show a Windows toast notification: {0}")]
    Notification(#[from] winrt_notification::Error),

    #[cfg(windows)]
    #[error("failed to resolve the current executable path: {0}")]
    CurrentExecutable(std::io::Error),

    #[cfg(windows)]
    #[error("failed to register the application for autostart: {0}")]
    Autostart(std::io::Error),

    #[cfg(windows)]
    #[error("failed to load a default Windows tray icon")]
    WindowsIconUnavailable,

    #[cfg(windows)]
    #[error("background worker thread panicked")]
    WorkerPanicked,

    #[error("application event channel closed unexpectedly")]
    EventChannelClosed,
}

#[derive(Debug, Error)]
pub enum WingetError {
    #[error("failed to run winget process: {0}")]
    Process(#[from] std::io::Error),

    #[error("winget exited unsuccessfully (code {code:?}): {stderr}")]
    CommandFailed { code: Option<i32>, stderr: String },

    #[error("winget output is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("failed to parse winget output: {0}")]
    Parse(#[from] ParseWingetError),
}

#[derive(Debug, Error)]
pub enum ParseWingetError {
    #[error("could not determine column positions from header line")]
    HeaderParseFailed,

    #[error("invalid package source: {0}")]
    InvalidSource(#[from] ParsePackageSourceError),
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("unexpected package source value: '{input}'")]
pub struct ParsePackageSourceError {
    pub input: String,
}
