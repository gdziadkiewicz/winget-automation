use std::process;

use crate::error::WingetError;

pub fn winget_update_raw() -> Result<String, WingetError> {
    run_command("winget", &["upgrade"])
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, WingetError> {
    let output = process::Command::new(cmd).args(args).output()?;
    let text = String::from_utf8(output.stdout)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_nonexistent_returns_process_error() {
        let result = run_command("__nonexistent_cmd_xyz__", &[]);
        assert!(
            matches!(result, Err(WingetError::Process(_))),
            "expected Process error, got: {:?}",
            result
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_run_command_echo() {
        let result = run_command("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[cfg(windows)]
    #[test]
    fn test_run_command_echo() {
        // On Windows, echo is a shell built-in; use cmd /C echo
        let result = run_command("cmd", &["/C", "echo", "hello"]);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }
}
