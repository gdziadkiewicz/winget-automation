use std::process::ExitCode;

use winget_automation::{error::WingetError, parser::parse_winget_raw, process::winget_update_raw};

fn main() -> Result<ExitCode, WingetError> {
    let raw = winget_update_raw()?;
    let packages = parse_winget_raw(&raw)?;
    for package in &packages {
        println!("{:?}", package);
    }
    Ok(ExitCode::SUCCESS)
}
