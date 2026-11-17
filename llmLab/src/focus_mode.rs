use crate::config::{
    load_keep_apps, load_session_snapshot, save_keep_apps, save_session_snapshot, ClosedAppEntry,
    KeepAppsStore, SessionSnapshot,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

pub const BACKGROUND_RAM_THRESHOLD_MB: u64 = 80;

/// Critical Windows OS, shell, audio, network/VPN, and terminal host processes
/// that must NEVER be shown as kill targets or terminated.
pub const PROTECTED_PROCESSES: &[&str] = &[
    // Core Windows kernel & session
    "system",
    "registry",
    "idle",
    "memory compression",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "services.exe",
    "winlogon.exe",
    "lsass.exe",
    "lsm.exe",
    "svchost.exe",
    "dwm.exe",
    "fontdrvhost.exe",
    "sihost.exe",
    "ctfmon.exe",
    "taskhostw.exe",
    "runtimebroker.exe",
    "wmiprvse.exe",
    "dllhost.exe",
    "spoolsv.exe",
    "dashost.exe",
    // Explorer & Shell
    "explorer.exe",
    "searchhost.exe",
    "searchindexer.exe",
    "startmenuexperiencehost.exe",
    "shellexperiencehost.exe",
    "textinputhost.exe",
    "lockapp.exe",
    "applicationframehost.exe",
    "systemsettings.exe",
    // Security & Defender
    "msmpeng.exe",
    "nissrv.exe",
    "securityhealthservice.exe",
    "securityhealthsystray.exe",
    // Network & VPN services
    "wlanext.exe",
    "wireguard.exe",
    "wg.exe",
    "openvpn.exe",
    "openvpn-gui.exe",
    "tailscaled.exe",
    "tailscale-ipn.exe",
    "nordvpn-service.exe",
    "protonvpnservice.exe",
    "v2ray.exe",
    "xray.exe",
    "sing-box.exe",
    "clash-verge.exe",
    "nekobox_core.exe",
    // Audio drivers & services
    "audiodg.exe",
    "rtkauduservice64.exe",
    "ravbg64.exe",
    "realtekaudioservice.exe",
    "nahimicservice.exe",
    "dolbydax2api.exe",
    "wavesmaxxaudio.exe",
    // Console hosts & llmLab itself
    "llmlab.exe",
    "conhost.exe",
    "openconsole.exe",
    "windowsterminal.exe",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRowStatus {
    Ready,
    Kept,
    Closing,
    HungTimeout,
    Closed,
}

impl ProcessRowStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Можно закрыть",
            Self::Kept => "Оставить",
            Self::Closing => "Закрытие...",
            Self::HungTimeout => "Завис (F=Kill)",
            Self::Closed => "Освобождено",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FocusTarget {
    pub pid: u32,
    pub process_name: String,
    pub window_title: String,
    pub exe_path: String,
    pub ram_mb: u64,
    pub hwnds: Vec<usize>,
    pub keep: bool,
    pub status: ProcessRowStatus,
}

#[derive(Debug, Default)]
pub struct FocusModeState {
    pub targets: Vec<FocusTarget>,
    pub selected_idx: usize,
    pub freed_mb_total: u64,
    pub pending_force_pids: Vec<u32>,
    pub last_status_msg: String,
    pub last_snapshot: SessionSnapshot,
}

pub fn is_protected_process(process_name: &str, protected_pids: &HashSet<u32>, pid: u32) -> bool {
    if protected_pids.contains(&pid) {
        return true;
    }
    let normalized = process_name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return true;
    }
    let with_exe = if normalized.ends_with(".exe") {
        normalized.clone()
    } else {
        format!("{normalized}.exe")
    };
    PROTECTED_PROCESSES
        .iter()
        .any(|&p| p == normalized || p == with_exe)
}

impl FocusModeState {
    pub fn refresh(&mut self) {
        let keep_store = load_keep_apps(None);
        let keep_set = keep_store.as_lowercase_set();
        self.last_snapshot = load_session_snapshot(None);

        let mut list = enumerate_focus_targets(&keep_set);
        list.sort_by(|a, b| b.ram_mb.cmp(&a.ram_mb));

        self.targets = list;
        if self.selected_idx >= self.targets.len() {
            self.selected_idx = self.targets.len().saturating_sub(1);
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(item) = self.targets.get_mut(self.selected_idx) {
            item.keep = !item.keep;
            item.status = if item.keep {
                ProcessRowStatus::Kept
            } else {
                ProcessRowStatus::Ready
            };
        }
    }

    pub fn toggle_all(&mut self) {
        let any_unkept = self.targets.iter().any(|t| !t.keep);
        for item in &mut self.targets {
            item.keep = any_unkept;
            item.status = if item.keep {
                ProcessRowStatus::Kept
            } else {
                ProcessRowStatus::Ready
            };
        }
    }

    pub fn persist_whitelist(&self) -> Result<()> {
        let mut names: Vec<String> = self
            .targets
            .iter()
            .filter(|t| t.keep)
            .map(|t| t.process_name.to_ascii_lowercase())
            .collect();
        names.sort();
        names.dedup();
        save_keep_apps(&KeepAppsStore { keep_processes: names }, None)
    }

    pub fn apply_cleanup(&mut self) {
        let _ = self.persist_whitelist();
        let to_close: Vec<usize> = self
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.keep && t.status != ProcessRowStatus::Closed)
            .map(|(i, _)| i)
            .collect();

        if to_close.is_empty() {
            self.last_status_msg = "Нет отмеченных к закрытию процессов.".to_string();
            return;
        }

        let mut snapshot_entries = Vec::new();
        for &idx in &to_close {
            let t = &mut self.targets[idx];
            if !t.exe_path.is_empty() {
                snapshot_entries.push(ClosedAppEntry {
                    process_name: t.process_name.clone(),
                    window_title: t.window_title.clone(),
                    exe_path: t.exe_path.clone(),
                    args: Vec::new(),
                    freed_mb: t.ram_mb,
                });
            }
            t.status = ProcessRowStatus::Closing;
            send_graceful_close(t.pid, &t.hwnds);
        }

        let snapshot = SessionSnapshot {
            created_at: Utc::now().to_rfc3339(),
            entries: snapshot_entries,
        };
        let _ = save_session_snapshot(&snapshot, None);
        self.last_snapshot = snapshot;

        // Wait up to 1.2s for clean exit of WM_CLOSE windows
        let deadline = Instant::now() + Duration::from_millis(1200);
        while Instant::now() < deadline {
            let still_running = to_close
                .iter()
                .any(|&idx| is_pid_alive(self.targets[idx].pid));
            if !still_running {
                break;
            }
            std::thread::sleep(Duration::from_millis(120));
        }

        let mut newly_freed = 0u64;
        self.pending_force_pids.clear();
        for &idx in &to_close {
            let t = &mut self.targets[idx];
            if is_pid_alive(t.pid) {
                t.status = ProcessRowStatus::HungTimeout;
                self.pending_force_pids.push(t.pid);
            } else {
                t.status = ProcessRowStatus::Closed;
                newly_freed += t.ram_mb;
            }
        }

        self.freed_mb_total += newly_freed;
        if self.pending_force_pids.is_empty() {
            self.last_status_msg = format!(
                "Освобождено +{:.2} ГБ RAM. Снапшот сохранён (Ctrl+R для возврата).",
                newly_freed as f64 / 1024.0
            );
        } else {
            self.last_status_msg = format!(
                "Освобождено +{:.2} ГБ RAM; {} проц. не ответили на WM_CLOSE (нажмите F для Kill).",
                newly_freed as f64 / 1024.0,
                self.pending_force_pids.len()
            );
        }
    }

    pub fn force_kill_pending(&mut self) {
        if self.pending_force_pids.is_empty() {
            self.last_status_msg = "Нет зависших процессов для принудительного завершения.".to_string();
            return;
        }
        let pids = std::mem::take(&mut self.pending_force_pids);
        let mut extra_freed = 0u64;
        for pid in pids {
            if force_terminate_pid(pid) {
                if let Some(t) = self.targets.iter_mut().find(|item| item.pid == pid) {
                    t.status = ProcessRowStatus::Closed;
                    extra_freed += t.ram_mb;
                }
            }
        }
        self.freed_mb_total += extra_freed;
        self.last_status_msg = format!(
            "Принудительно завершено: +{:.2} ГБ RAM.",
            extra_freed as f64 / 1024.0
        );
    }

    pub fn restore_closed_apps(&mut self) -> usize {
        let snapshot = load_session_snapshot(None);
        if snapshot.entries.is_empty() {
            self.last_status_msg = "Нет закрытых приложений в последнем снапшоте.".to_string();
            return 0;
        }
        let mut launched = 0usize;
        let mut seen_paths = HashSet::new();
        for entry in &snapshot.entries {
            if entry.exe_path.is_empty()
                || !seen_paths.insert(entry.exe_path.to_ascii_lowercase())
                || !Path::new(&entry.exe_path).exists()
            {
                continue;
            }
            if Command::new(&entry.exe_path).args(&entry.args).spawn().is_ok() {
                launched += 1;
            }
        }
        self.last_status_msg = format!("Восстановлено приложений: {launched}");
        launched
    }
}

fn enumerate_focus_targets(keep_set: &HashSet<String>) -> Vec<FocusTarget> {
    #[cfg(windows)]
    {
        windows_impl::enumerate_targets(keep_set)
    }

    #[cfg(not(windows))]
    {
        let _ = keep_set;
        Vec::new()
    }
}

fn send_graceful_close(pid: u32, hwnds: &[usize]) {
    #[cfg(windows)]
    {
        windows_impl::post_wm_close(pid, hwnds);
    }
    #[cfg(not(windows))]
    {
        let _ = (pid, hwnds);
    }
}

fn is_pid_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        windows_impl::check_pid_alive(pid)
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        false
    }
}

fn force_terminate_pid(pid: u32) -> bool {
    #[cfg(windows)]
    {
        windows_impl::terminate_process(pid)
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        false
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::ffi::c_void;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, TRUE, WPARAM};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, GetExitCodeProcess, OpenProcess, QueryFullProcessImageNameW,
        TerminateProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, PostMessageW, GWL_EXSTYLE, GW_OWNER, WM_CLOSE,
        WS_EX_TOOLWINDOW,
    };

    const STILL_ACTIVE_CODE: u32 = 259;

    struct WindowCollectState {
        windows_by_pid: HashMap<u32, Vec<(usize, String)>>,
    }

    unsafe extern "system" fn enum_window_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }
        // Skip tool windows & owned popups
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
            return TRUE;
        }
        if GetWindow(hwnd, GW_OWNER).is_ok_and(|owner| !owner.0.is_null()) {
            return TRUE;
        }

        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return TRUE;
        }
        let mut buf = vec![0u16; (len as usize) + 1];
        let copied = GetWindowTextW(hwnd, &mut buf);
        if copied <= 0 {
            return TRUE;
        }
        let title = String::from_utf16_lossy(&buf[..copied as usize])
            .trim()
            .to_string();
        if title.is_empty() {
            return TRUE;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return TRUE;
        }

        let state = &mut *(lparam.0 as *mut WindowCollectState);
        state
            .windows_by_pid
            .entry(pid)
            .or_default()
            .push((hwnd.0 as usize, title));
        TRUE
    }

    fn collect_protected_ancestor_pids(parent_map: &HashMap<u32, u32>) -> HashSet<u32> {
        let mut protected = HashSet::new();
        let mut current = unsafe { GetCurrentProcessId() };
        for _ in 0..12 {
            if current == 0 || !protected.insert(current) {
                break;
            }
            match parent_map.get(&current) {
                Some(&ppid) if ppid != 0 => current = ppid,
                _ => break,
            }
        }
        protected
    }

    fn query_exe_and_ram(pid: u32) -> (String, u64) {
        let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) })
        else {
            return (String::new(), 0);
        };

        let mut exe_path = String::new();
        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        if unsafe {
            QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len)
        }
        .is_ok()
        {
            exe_path = String::from_utf16_lossy(&buf[..len as usize]);
        }

        let mut pmc = PROCESS_MEMORY_COUNTERS {
            cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        let ram_mb = if unsafe { GetProcessMemoryInfo(handle, &mut pmc, pmc.cb) }.is_ok() {
            (pmc.WorkingSetSize as u64) / (1024 * 1024)
        } else {
            0
        };

        let _ = unsafe { CloseHandle(handle) };
        (exe_path, ram_mb)
    }

    pub fn enumerate_targets(keep_set: &HashSet<String>) -> Vec<FocusTarget> {
        let mut win_state = WindowCollectState {
            windows_by_pid: HashMap::new(),
        };
        unsafe {
            let _ = EnumWindows(
                Some(enum_window_cb),
                LPARAM(&mut win_state as *mut WindowCollectState as isize),
            );
        }

        let Ok(snap) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
            return Vec::new();
        };

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut processes = Vec::new();
        let mut parent_map = HashMap::new();

        if unsafe { Process32FirstW(snap, &mut entry) }.is_ok() {
            loop {
                let end = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
                parent_map.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                processes.push((entry.th32ProcessID, name));

                if unsafe { Process32NextW(snap, &mut entry) }.is_err() {
                    break;
                }
            }
        }
        let _ = unsafe { CloseHandle(snap) };

        let protected_pids = collect_protected_ancestor_pids(&parent_map);
        let mut targets = Vec::new();

        for (pid, process_name) in processes {
            if is_protected_process(&process_name, &protected_pids, pid) {
                continue;
            }
            let (exe_path, ram_mb) = query_exe_and_ram(pid);
            let win_list = win_state.windows_by_pid.remove(&pid).unwrap_or_default();

            if win_list.is_empty() && ram_mb < BACKGROUND_RAM_THRESHOLD_MB {
                continue;
            }

            let window_title = if let Some((_, first_title)) = win_list.first() {
                first_title.clone()
            } else {
                format!("[Фоновый процесс] {process_name}")
            };
            let hwnds: Vec<usize> = win_list.into_iter().map(|(h, _)| h).collect();

            let lower = process_name.to_ascii_lowercase();
            let keep = keep_set.contains(&lower);
            let status = if keep {
                ProcessRowStatus::Kept
            } else {
                ProcessRowStatus::Ready
            };

            targets.push(FocusTarget {
                pid,
                process_name,
                window_title,
                exe_path,
                ram_mb,
                hwnds,
                keep,
                status,
            });
        }

        targets
    }

    pub fn post_wm_close(pid: u32, hwnds: &[usize]) {
        if !hwnds.is_empty() {
            for &raw in hwnds {
                let hwnd = HWND(raw as *mut c_void);
                unsafe {
                    let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
            }
            return;
        }
        // Background process without a visible window: no HWND to post WM_CLOSE to;
        // leave for user confirmation via Kill (`F`) if still running after timeout.
        let _ = pid;
    }

    pub fn check_pid_alive(pid: u32) -> bool {
        let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) })
        else {
            return false;
        };
        let mut exit_code = 0u32;
        let alive = unsafe { GetExitCodeProcess(handle, &mut exit_code) }.is_ok()
            && exit_code == STILL_ACTIVE_CODE;
        let _ = unsafe { CloseHandle(handle) };
        alive
    }

    pub fn terminate_process(pid: u32) -> bool {
        let Ok(handle) = (unsafe { OpenProcess(PROCESS_TERMINATE, false, pid) }) else {
            return false;
        };
        let ok = unsafe { TerminateProcess(handle, 1) }.is_ok();
        let _ = unsafe { CloseHandle(handle) };
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_windows_processes_are_protected() {
        let pids = HashSet::from([42u32]);
        assert!(is_protected_process("System", &pids, 4));
        assert!(is_protected_process("CSRSS.EXE", &pids, 500));
        assert!(is_protected_process("explorer.exe", &pids, 1000));
        assert!(is_protected_process("SearchHost.exe", &pids, 1001));
        assert!(is_protected_process("StartMenuExperienceHost.exe", &pids, 1002));
        assert!(is_protected_process("wireguard.exe", &pids, 1003));
        assert!(is_protected_process("RtkAudUService64.exe", &pids, 1004));
        assert!(is_protected_process("llmLab.exe", &pids, 1005));
        // Ancestor PID protection (e.g. parent terminal)
        assert!(is_protected_process("powershell.exe", &pids, 42));
    }

    #[test]
    fn user_applications_are_not_protected() {
        let pids = HashSet::from([42u32]);
        assert!(!is_protected_process("chrome.exe", &pids, 8888));
        assert!(!is_protected_process("Discord.exe", &pids, 8889));
        assert!(!is_protected_process("Code.exe", &pids, 8890));
    }
}
