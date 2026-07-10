#[cfg(not(windows))]
use std::process::ExitCode;

#[cfg(not(windows))]
use crate::error::AppError;

#[cfg(not(windows))]
pub fn run() -> Result<ExitCode, AppError> {
    eprintln!(
        "winget-automation runs as a Windows tray application and is not supported on this platform."
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(windows)]
mod imp {
    use std::{
        env,
        process::ExitCode,
        sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
        thread::{self, JoinHandle},
        time::Duration as StdDuration,
    };

    use tray_item::{IconSource, TrayItem};
    use windows_sys::Win32::UI::WindowsAndMessaging::{IDI_APPLICATION, LoadIconW};
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    use winrt_notification::{Duration as ToastDuration, Sound, Toast};

    use crate::{
        error::{AppError, WingetError},
        parser::{WingetUpdateOutput, parse_winget_raw},
        process::winget_update_raw,
    };

    const APP_NAME: &str = "winget-automation";
    const AUTOSTART_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const CHECK_INTERVAL_HOURS: u64 = 4;
    const CHECK_INTERVAL: StdDuration = StdDuration::from_secs(CHECK_INTERVAL_HOURS * 60 * 60);
    const TOAST_PACKAGE_LIMIT: usize = 3;

    enum AppEvent {
        CheckNow,
        Quit,
        Report(CheckReport),
    }

    enum WorkerCommand {
        CheckNow,
        Quit,
    }

    enum CheckTrigger {
        Startup,
        Manual,
        Scheduled,
    }

    struct CheckReport {
        trigger: CheckTrigger,
        result: Result<Vec<WingetUpdateOutput>, WingetError>,
    }

    pub fn run() -> Result<ExitCode, AppError> {
        ensure_autostart()?;

        let (event_tx, event_rx) = mpsc::channel();
        let (worker_tx, worker_rx) = mpsc::channel();
        let worker = spawn_worker(event_tx.clone(), worker_rx);

        let mut tray = build_tray(event_tx)?;
        let status_id = tray
            .inner_mut()
            .add_label_with_id("Status: checking for updates...")?;

        loop {
            match event_rx.recv().map_err(|_| AppError::EventChannelClosed)? {
                AppEvent::CheckNow => {
                    tray.inner_mut()
                        .set_label("Status: checking for updates...", status_id)?;
                    tray.set_icon(default_icon()?)?;
                    let _ = worker_tx.send(WorkerCommand::CheckNow);
                }
                AppEvent::Quit => {
                    let _ = worker_tx.send(WorkerCommand::Quit);
                    break;
                }
                AppEvent::Report(report) => {
                    let status = handle_report(&mut tray, &report)?;
                    tray.inner_mut().set_label(&status, status_id)?;
                }
            }
        }

        worker.join().map_err(|_| AppError::WorkerPanicked)?;
        Ok(ExitCode::SUCCESS)
    }

    fn build_tray(event_tx: Sender<AppEvent>) -> Result<TrayItem, AppError> {
        let mut tray = TrayItem::new(APP_NAME, default_icon()?)?;
        tray.add_menu_item("Check now", {
            let event_tx = event_tx.clone();
            move || {
                let _ = event_tx.send(AppEvent::CheckNow);
            }
        })?;
        tray.add_menu_item("Quit", move || {
            let _ = event_tx.send(AppEvent::Quit);
        })?;
        Ok(tray)
    }

    fn default_icon() -> Result<IconSource, AppError> {
        let icon = unsafe { LoadIconW(0, IDI_APPLICATION) };
        if icon == 0 {
            return Err(AppError::WindowsIconUnavailable);
        }
        Ok(IconSource::RawIcon(icon))
    }

    fn ensure_autostart() -> Result<(), AppError> {
        let executable = env::current_exe().map_err(AppError::CurrentExecutable)?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run_key, _) = hkcu
            .create_subkey(AUTOSTART_KEY_PATH)
            .map_err(AppError::Autostart)?;
        run_key
            .set_value(APP_NAME, &format!("\"{}\"", executable.display()))
            .map_err(AppError::Autostart)?;
        Ok(())
    }

    fn spawn_worker(
        event_tx: Sender<AppEvent>,
        command_rx: Receiver<WorkerCommand>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut next_trigger = CheckTrigger::Startup;

            loop {
                let result =
                    winget_update_raw().and_then(|raw| parse_winget_raw(&raw).map_err(Into::into));
                if event_tx
                    .send(AppEvent::Report(CheckReport {
                        trigger: next_trigger,
                        result,
                    }))
                    .is_err()
                {
                    break;
                }

                match command_rx.recv_timeout(CHECK_INTERVAL) {
                    Ok(WorkerCommand::CheckNow) => next_trigger = CheckTrigger::Manual,
                    Ok(WorkerCommand::Quit) | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => next_trigger = CheckTrigger::Scheduled,
                }
            }
        })
    }

    fn handle_report(tray: &mut TrayItem, report: &CheckReport) -> Result<String, AppError> {
        match &report.result {
            Ok(packages) if packages.is_empty() => {
                tray.inner_mut()
                    .set_tooltip("winget-automation: no package updates available")?;
                if matches!(report.trigger, CheckTrigger::Manual) {
                    show_toast(
                        "No package updates available",
                        "winget did not report any outdated packages.",
                        "winget-automation will keep checking in the background.",
                    )?;
                }
                Ok("Status: up to date".to_string())
            }
            Ok(packages) => {
                let title = format!("{} available", update_count_label(packages.len()));
                let line1 = summarize_packages(packages);
                let line2 = if packages.len() > TOAST_PACKAGE_LIMIT {
                    format!(
                        "and {} more package(s)",
                        packages.len() - TOAST_PACKAGE_LIMIT
                    )
                } else {
                    "Use winget upgrade to install the latest versions.".to_string()
                };
                tray.inner_mut()
                    .set_tooltip(&format!("winget-automation: {}", title))?;
                show_toast(&title, &line1, &line2)?;
                Ok(format!("Status: {}", title))
            }
            Err(error) => {
                let status = format!("Status: last check failed ({error})");
                tray.inner_mut().set_tooltip(&status)?;
                if matches!(report.trigger, CheckTrigger::Manual) {
                    show_toast(
                        "Package check failed",
                        &error.to_string(),
                        "See the tray menu for the latest status.",
                    )?;
                }
                Ok(status)
            }
        }
    }

    fn show_toast(title: &str, line1: &str, line2: &str) -> Result<(), AppError> {
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(line1)
            .text2(line2)
            .duration(ToastDuration::Short)
            .sound(Some(Sound::Default))
            .show()?;
        Ok(())
    }

    fn update_count_label(count: usize) -> String {
        match count {
            1 => format!("{count} package update is"),
            n => format!("{n} package updates are"),
        }
    }

    fn summarize_packages(packages: &[WingetUpdateOutput]) -> String {
        packages
            .iter()
            .take(TOAST_PACKAGE_LIMIT)
            .map(|package| package.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(windows)]
pub use imp::run;
