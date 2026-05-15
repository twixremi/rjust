use anyhow::Result;

pub fn initialize() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        crate::log("[security] !!! WARNING: Experimental Sandbox Active !!!");
        crate::log("[security] This feature may cause crashes if Minecraft performs unusual syscalls.");
        setup_seccomp()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        crate::log("[security] Windows sandbox not implemented. Using system defaults.");
    }
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_seccomp() -> Result<()> {
    use libseccomp::*;
    
    println!("[rjust] Setting up libseccomp sandbox (Linux)");

    let mut filter = ScmpFilterContext::new_filter(ScmpAction::Allow)?;

    // Blacklist dangerous syscalls for modules
    filter.add_rule(ScmpAction::KillProcess, ScmpSyscall::from_name("execve")?)?;
    filter.add_rule(ScmpAction::KillProcess, ScmpSyscall::from_name("ptrace")?)?;

    filter.load()?;
    Ok(())
}
