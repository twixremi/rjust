use anyhow::Result;

#[cfg(target_os = "windows")]
pub fn initialize() -> Result<()> {
    use windows_sys::Win32::System::Console::AllocConsole;
    
    unsafe {
        if AllocConsole() != 0 {
            println!("[rjust] Windows Debug Console allocated.");
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn initialize() -> Result<()> {
    use std::process::Command;
    
    println!("[rjust] Initializing Linux Debug Console...");
    
    // Attempt to spawn a terminal that could display logs
    // In a production environment, we'd pipe stdout to a FIFO and tail it in the terminal
    let _ = Command::new("xterm")
        .arg("-title")
        .arg("rjust Debug Console")
        .arg("-e")
        .arg("echo 'rjust Debug Console active. Logs will appear here.'; sleep 10")
        .spawn();

    Ok(())
}
