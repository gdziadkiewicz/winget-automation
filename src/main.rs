use std::process::ExitCode;

use winget_automation::{app, error::AppError};

fn main() -> Result<ExitCode, AppError> {
    app::run()
}
