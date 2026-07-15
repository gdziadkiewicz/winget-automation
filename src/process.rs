use std::process;

use crate::error::WingetError;

pub fn winget_update_raw() -> Result<String, WingetError> {
    run_command("winget", &["upgrade"])
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, WingetError> {
    let output = process::Command::new(cmd).args(args).output()?;
    if !output.status.success() {
        return Err(WingetError::CommandFailed {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
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

    #[cfg(windows)]
    #[test]
    fn test_run_command_nonzero_exit_is_error() {
        let result = run_command("cmd", &["/C", "echo failure 1>&2 & exit /B 7"]);
        assert!(matches!(
            result,
            Err(WingetError::CommandFailed {
                code: Some(7),
                stderr
            }) if stderr.contains("failure")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_command_nonzero_exit_is_error() {
        let result = run_command("sh", &["-c", "echo failure >&2; exit 7"]);
        assert!(matches!(
            result,
            Err(WingetError::CommandFailed {
                code: Some(7),
                stderr
            }) if stderr.contains("failure")
        ));
    }
}
