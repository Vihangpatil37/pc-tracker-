use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivitySample {
    pub exe_name: String,
    pub window_title: Option<String>,
    pub pid: u32,
    pub timestamp: i64,
}

pub fn get_foreground_window_info() -> Option<(HWND, u32)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        Some((hwnd, pid))
    }
}

pub fn get_window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let len = GetWindowTextLengthA(hwnd);
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u8; (len + 1) as usize];
        let actual = GetWindowTextA(hwnd, &mut buf);
        if actual == 0 {
            return None;
        }
        buf.truncate(actual as usize);
        String::from_utf8(buf).ok()
    }
}

pub fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
        if handle.is_invalid() {
            return None;
        }
        let mut buf = [0u8; 260];
        let mut size = buf.len() as u32;
        let path = windows::core::PSTR(buf.as_mut_ptr());
        let result = QueryFullProcessImageNameA(handle, PROCESS_NAME_WIN32, path, &mut size);
        let _ = CloseHandle(handle);
        if result.is_err() {
            return None;
        }
        let end = buf.iter().position(|&b| b == 0).unwrap_or(size as usize);
        let path_str = String::from_utf8_lossy(&buf[..end]).to_string();
        std::path::Path::new(&path_str)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
    }
}

pub fn sample_current() -> Option<ActivitySample> {
    let (hwnd, pid) = get_foreground_window_info()?;
    let exe_name = get_process_name(pid)?;
    let window_title = get_window_title(hwnd);
    let timestamp = chrono::Utc::now().timestamp();

    Some(ActivitySample {
        exe_name,
        window_title,
        pid,
        timestamp,
    })
}
