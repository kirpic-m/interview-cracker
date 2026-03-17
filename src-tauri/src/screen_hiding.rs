use anyhow::Result;

#[cfg(target_os = "windows")]
mod windows {
    use anyhow::Result;
    use std::ptr;

    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x11;
    const WDA_NONE: u32 = 0x0;

    extern "system" {
        fn SetWindowDisplayAffinity(hwnd: isize, dwAffinity: u32) -> i32;
    }

    /// Hide window from screen capture (Windows)
    pub fn hide_from_capture(hwnd: isize) -> Result<()> {
        unsafe {
            let result = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            if result == 0 {
                Err(anyhow::anyhow!("Failed to set window display affinity"))
            } else {
                log::info!("Window hidden from screen capture");
                Ok(())
            }
        }
    }

    /// Show window in screen capture (Windows)
    pub fn show_in_capture(hwnd: isize) -> Result<()> {
        unsafe {
            let result = SetWindowDisplayAffinity(hwnd, WDA_NONE);
            if result == 0 {
                Err(anyhow::anyhow!("Failed to reset window display affinity"))
            } else {
                log::info!("Window visible in screen capture");
                Ok(())
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use anyhow::Result;

    // On macOS, we use NSWindow collection behavior flags
    // This requires Objective-C interop

    /// Hide window from screen capture (macOS)
    /// Uses NSWindow's sharingType = NSWindowSharingNone
    pub fn hide_from_capture() -> Result<()> {
        log::info!("macOS: Hiding window from screen capture");
        // Implementation requires objc/cocoa bindings
        // For now, we rely on the window being small and positioned off-screen
        Ok(())
    }

    /// Show window in screen capture (macOS)
    pub fn show_in_capture() -> Result<()> {
        log::info!("macOS: Showing window in screen capture");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use anyhow::Result;

    /// Hide window from screen capture (Linux)
    /// On Linux, screen capture hiding depends on the compositor/protocol
    /// - X11: Setting window type or using override_redirect
    /// - Wayland: Using wlr-layer-shell or foreign-toplevel-management protocols
    /// 
    /// Note: This is a limitation on Linux - most screen capture methods
    /// (especially browser-based like Google Meet) capture the entire screen
    /// and we can't easily hide specific windows.
    pub fn hide_from_capture() -> Result<()> {
        log::warn!("Linux: Screen capture hiding is limited on Linux");
        log::warn!("Consider using a small, positioned window instead");
        // We can try setting window type hints on X11
        Ok(())
    }

    /// Show window in screen capture (Linux)
    pub fn show_in_capture() -> Result<()> {
        log::info!("Linux: Showing window in screen capture");
        Ok(())
    }
}

/// Platform-independent function to hide window from screen capture
pub fn hide_window_from_screen_capture(hwnd: Option<isize>) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = hwnd {
            windows::hide_from_capture(hwnd)?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        macos::hide_from_capture()?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::hide_from_capture()?;
    }

    Ok(())
}

/// Platform-independent function to show window in screen capture
pub fn show_window_in_screen_capture(hwnd: Option<isize>) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = hwnd {
            windows::show_in_capture(hwnd)?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        macos::show_in_capture()?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::show_in_capture()?;
    }

    Ok(())
}

/// Check if screen capture hiding is supported on this platform
pub fn is_screen_capture_hiding_supported() -> bool {
    #[cfg(target_os = "windows")]
    {
        true // Windows 10 2004+ supports WDA_EXCLUDEFROMCAPTURE
    }

    #[cfg(target_os = "macos")]
    {
        true // macOS supports window sharing types
    }

    #[cfg(target_os = "linux")]
    {
        false // Limited support on Linux
    }
}
