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
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateIconFromResourceEx, LR_DEFAULTCOLOR, MB_ICONINFORMATION, MB_OK, MessageBoxW,
    };
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    use winrt_notification::{Duration as ToastDuration, Sound, Toast};

    use crate::{
        error::{AppError, WingetError},
        parser::{PackageSource, WingetUpdateOutput, parse_winget_raw},
        process::winget_update_raw,
    };

    const APP_NAME: &str = "WinGet Update Monitor";
    const APP_USER_MODEL_ID: &str = "gdziadkiewicz.WinGetUpdateMonitor";
    const AUTOSTART_VALUE_NAME: &str = "winget-automation";
    const AUTOSTART_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const CHECK_INTERVAL_HOURS: u64 = 4;
    const CHECK_INTERVAL: StdDuration = StdDuration::from_secs(CHECK_INTERVAL_HOURS * 60 * 60);
    const TOAST_PACKAGE_LIMIT: usize = 3;
    const TRAY_ICON_PNG: &[u8] = include_bytes!("../assets/tray-icon.png");

    enum AppEvent {
        CheckNow,
        ShowUpdates,
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

        let (mut tray, status_id) = build_tray(event_tx)?;
        let mut latest_updates = Vec::new();

        loop {
            match event_rx.recv().map_err(|_| AppError::EventChannelClosed)? {
                AppEvent::CheckNow => {
                    tray.inner_mut()
                        .set_label("Checking for updates…", status_id)?;
                    tray.set_icon(default_icon()?)?;
                    let _ = worker_tx.send(WorkerCommand::CheckNow);
                }
                AppEvent::ShowUpdates => {
                    show_updates_window(&latest_updates)?;
                }
                AppEvent::Quit => {
                    let _ = worker_tx.send(WorkerCommand::Quit);
                    break;
                }
                AppEvent::Report(report) => {
                    let status = handle_report(&mut tray, &report)?;
                    latest_updates = report
                        .result
                        .as_ref()
                        .map_or_else(|_| Vec::new(), Clone::clone);
                    tray.inner_mut().set_label(&status, status_id)?;
                }
            }
        }

        worker.join().map_err(|_| AppError::WorkerPanicked)?;
        Ok(ExitCode::SUCCESS)
    }

    fn build_tray(event_tx: Sender<AppEvent>) -> Result<(TrayItem, u32), AppError> {
        let mut tray = TrayItem::new(APP_NAME, default_icon()?)?;
        let status_id = tray
            .inner_mut()
            .add_label_with_id("Checking for updates…")?;
        tray.inner_mut().add_separator()?;
        tray.add_menu_item("Show available updates", {
            let event_tx = event_tx.clone();
            move || {
                let _ = event_tx.send(AppEvent::ShowUpdates);
            }
        })?;
        tray.add_menu_item("Check for updates now", {
            let event_tx = event_tx.clone();
            move || {
                let _ = event_tx.send(AppEvent::CheckNow);
            }
        })?;
        tray.inner_mut().add_separator()?;
        tray.add_menu_item("Exit", move || {
            let _ = event_tx.send(AppEvent::Quit);
        })?;
        Ok((tray, status_id))
    }

    fn default_icon() -> Result<IconSource, AppError> {
        let icon = unsafe {
            CreateIconFromResourceEx(
                TRAY_ICON_PNG.as_ptr(),
                TRAY_ICON_PNG.len() as u32,
                1,
                0x0003_0000,
                0,
                0,
                LR_DEFAULTCOLOR,
            )
        };
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
            .set_value(
                AUTOSTART_VALUE_NAME,
                &format!("\"{}\"", executable.display()),
            )
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
                    .set_tooltip("WinGet Update Monitor — you're up to date")?;
                if matches!(report.trigger, CheckTrigger::Manual) {
                    show_toast(
                        "You're up to date",
                        "No package updates were found.",
                        "We'll check again automatically in 4 hours.",
                    )?;
                }
                Ok("Up to date".to_string())
            }
            Ok(packages) => {
                let title = update_count_label(packages.len());
                let line1 = summarize_packages(packages);
                let line2 = if packages.len() > TOAST_PACKAGE_LIMIT {
                    format!("Plus {} more", packages.len() - TOAST_PACKAGE_LIMIT)
                } else {
                    "Right-click the tray icon to view details.".to_string()
                };
                tray.inner_mut()
                    .set_tooltip(&format!("WinGet Update Monitor — {title}"))?;
                show_toast(&title, &line1, &line2)?;
                Ok(title)
            }
            Err(error) => {
                let status = "Last check failed".to_string();
                tray.inner_mut()
                    .set_tooltip("WinGet Update Monitor — last check failed")?;
                if matches!(report.trigger, CheckTrigger::Manual) {
                    show_toast(
                        "Update check failed",
                        &error.to_string(),
                        "Check your connection and try again.",
                    )?;
                }
                Ok(status)
            }
        }
    }

    fn show_updates_window(packages: &[WingetUpdateOutput]) -> Result<(), AppError> {
        let body = update_window_body(packages);
        let title = to_wide(APP_NAME);
        let body = to_wide(&body);
        unsafe {
            MessageBoxW(0, body.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
        }
        Ok(())
    }

    fn update_window_body(packages: &[WingetUpdateOutput]) -> String {
        if packages.is_empty() {
            return "No package updates are currently available. Use Check for updates now to refresh."
                .to_string();
        }

        let mut lines = vec![update_count_label(packages.len()), String::new()];
        lines.extend(packages.iter().map(|package| {
            format!(
                "{}\n  {}: {} -> {} ({})",
                package.name,
                package.id,
                package.version,
                package.available,
                package.source.as_str()
            )
        }));
        lines.push(String::new());
        lines.push("Run winget upgrade in a terminal to install these updates.".to_string());
        lines.join("\n")
    }

    fn to_wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn show_toast(title: &str, line1: &str, line2: &str) -> Result<(), AppError> {
        if toast(APP_USER_MODEL_ID, title, line1, line2)
            .show()
            .is_err()
        {
            toast(Toast::POWERSHELL_APP_ID, title, line1, line2).show()?;
        }
        Ok(())
    }

    fn toast(app_id: &str, title: &str, line1: &str, line2: &str) -> Toast {
        Toast::new(app_id)
            .title(title)
            .text1(line1)
            .text2(line2)
            .duration(ToastDuration::Short)
            .sound(Some(Sound::Default))
    }

    fn update_count_label(count: usize) -> String {
        match count {
            1 => "1 app update available".to_string(),
            n => format!("{n} app updates available"),
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

    impl PackageSource {
        fn as_str(&self) -> &'static str {
            match self {
                PackageSource::WinGet => "winget",
                PackageSource::MsStore => "msstore",
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn embedded_tray_icon_is_valid() {
            let IconSource::RawIcon(icon) = default_icon().expect("icon should load") else {
                panic!("expected an embedded raw icon");
            };
            assert_ne!(icon, 0);
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon) };
        }

        #[test]
        fn update_count_labels_are_natural() {
            assert_eq!(update_count_label(1), "1 app update available");
            assert_eq!(update_count_label(4), "4 app updates available");
        }
    }
}

#[cfg(windows)]
pub use imp::run;
