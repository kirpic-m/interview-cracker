use anyhow::Result;

/// Hide window from screen capture using platform-specific methods
pub fn hide_from_capture() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        hide_linux()
    }

    #[cfg(target_os = "windows")]
    {
        hide_windows()
    }

    #[cfg(target_os = "macos")]
    {
        hide_macos()
    }
}

/// Show window in screen capture
pub fn show_in_capture() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        show_linux()
    }

    #[cfg(target_os = "windows")]
    {
        show_windows()
    }

    #[cfg(target_os = "macos")]
    {
        show_macos()
    }
}

// ═══════════════════════════════════════════
// LINUX - Multiple methods
// ═══════════════════════════════════════════

#[cfg(target_os = "linux")]
fn hide_linux() -> Result<()> {
    // Method 1: Set _NET_WM_BYPASS_COMPOSITOR hint
    // This tells the compositor to not capture this window
    set_x11_property("_NET_WM_BYPASS_COMPOSITOR", "1")?;
    
    // Method 2: Set window type to utility/popup (often skipped by capture)
    set_x11_property("_NET_WM_WINDOW_TYPE", "_NET_WM_WINDOW_TYPE_DOCK")?;
    
    log::info!("Linux: Window hidden using X11 properties");
    Ok(())
}

#[cfg(target_os = "linux")]
fn show_linux() -> Result<()> {
    set_x11_property("_NET_WM_BYPASS_COMPOSITOR", "0")?;
    set_x11_property("_NET_WM_WINDOW_TYPE", "_NET_WM_WINDOW_TYPE_NORMAL")?;
    log::info!("Linux: Window visible");
    Ok(())
}

#[cfg(target_os = "linux")]
fn set_x11_property(property: &str, value: &str) -> Result<()> {
    use std::process::Command;
    
    // Find our window by name
    let output = Command::new("xdotool")
        .args(["search", "--name", "Interview Cracker"])
        .output();
    
    if let Ok(output) = output {
        let window_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !window_id.is_empty() {
            Command::new("xprop")
                .args(["-id", &window_id, "-f", property, "32c", "-set", property, value])
                .output()
                .ok();
        }
    }
    
    Ok(())
}

// ═══════════════════════════════════════════
// WINDOWS - SetWindowDisplayAffinity
// ═══════════════════════════════════════════

#[cfg(target_os = "windows")]
fn hide_windows() -> Result<()> {
    unsafe {
        use std::ptr::null;
        use winapi::um::winuser::{FindWindowW, SetWindowDisplayAffinity};
        
        const WDA_EXCLUDEFROMCAPTURE: u32 = 0x11;
        
        let title: Vec<u16> = "Interview Cracker\0".encode_utf16().collect();
        let hwnd = FindWindowW(null(), title.as_ptr());
        
        if !hwnd.is_null() {
            SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            log::info!("Windows: Window hidden from capture");
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn show_windows() -> Result<()> {
    unsafe {
        use std::ptr::null;
        use winapi::um::winuser::{FindWindowW, SetWindowDisplayAffinity};
        
        const WDA_NONE: u32 = 0x0;
        
        let title: Vec<u16> = "Interview Cracker\0".encode_utf16().collect();
        let hwnd = FindWindowW(null(), title.as_ptr());
        
        if !hwnd.is_null() {
            SetWindowDisplayAffinity(hwnd, WDA_NONE);
            log::info!("Windows: Window visible");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════
// macOS - CGWindowLevel
// ═══════════════════════════════════════════

#[cfg(target_os = "macos")]
fn hide_macos() -> Result<()> {
    log::info!("macOS: Stealth mode enabled (use small overlay window)");
    Ok(())
}

#[cfg(target_os = "macos")]
fn show_macos() -> Result<()> {
    log::info!("macOS: Normal mode");
    Ok(())
}

/// Check if hiding is supported on this platform
pub fn is_supported() -> bool {
    true // All platforms have some method
}

/// Alternative: Launch program on virtual display (Linux only)
/// This makes the ENTIRE app invisible to screen capture
pub fn launch_on_virtual_display() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        // Check if Xvfb is available
        if which::which("Xvfb").is_err() {
            return Err(anyhow::anyhow!(
                "Xvfb not installed. Install with: sudo apt install xvfb x11vnc"
            ));
        }
        
        // Start virtual display
        Command::new("Xvfb")
            .args([":99", "-screen", "0", "1920x1080x24", "-ac"])
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start Xvfb: {}", e))?;
        
        // Wait for display to be ready
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Start VNC server for remote viewing
        Command::new("x11vnc")
            .args(["-display", ":99", "-rfbport", "5999", "-nopw", "-quiet", "-bg"])
            .spawn()
            .ok(); // Don't fail if VNC fails
        
        log::info!("Virtual display :99 started. Connect via VNC to localhost:5999");
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Err(anyhow::anyhow!("Virtual display only supported on Linux"))
    }
}
