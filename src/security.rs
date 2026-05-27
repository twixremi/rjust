use anyhow::Result;

pub fn initialize() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        crate::log("[security] !!! WARNING: Robust Linux Sandbox Active !!!");
        setup_seccomp()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        crate::log("[security] !!! WARNING: Windows Low Integrity Sandbox Active !!!");
        setup_windows_sandbox()?;
    }

    #[cfg(target_os = "macos")]
    {
        crate::log("[security] !!! WARNING: macOS Seatbelt Sandbox Active !!!");
        setup_macos_sandbox()?;
    }
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_seccomp() -> Result<()> {
    use libseccomp::*;
    
    println!("[rjust] Setting up libseccomp sandbox (Linux)");

    let mut filter = ScmpFilterContext::new_filter(ScmpAction::Allow)?;

    // Blacklist dangerous syscalls for process orchestration, networking, debugging, and modules
    let dangerous_syscalls = [
        "execve", "execveat", "fork", "vfork",
        "socket", "connect", "bind", "listen", "accept", "accept4",
        "ptrace", "reboot", "init_module", "finit_module",
        "delete_module", "kexec_load", "kexec_file_load"
    ];
    for syscall in dangerous_syscalls {
        if let Ok(sys) = ScmpSyscall::from_name(syscall) {
            filter.add_rule(ScmpAction::KillProcess, sys)?;
        }
    }

    filter.load()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn setup_windows_sandbox() -> Result<()> {
    use windows_sys::Win32::Security::{
        AdjustTokenPrivileges, SetTokenInformation, AllocateAndInitializeSid, FreeSid,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, TOKEN_ADJUST_DEFAULT,
        TokenIntegrityLevel, TOKEN_MANDATORY_LABEL, SID_AND_ATTRIBUTES,
        SE_GROUP_INTEGRITY, SID_IDENTIFIER_AUTHORITY
    };
    use windows_sys::Win32::System::Threading::{OpenProcessToken, GetCurrentProcess};
    use windows_sys::Win32::Foundation::{HANDLE, CloseHandle};

    unsafe {
        let process_handle = GetCurrentProcess();
        let mut token_handle: HANDLE = 0;
        
        // Open the current process token with duplicate, adjust, query and adjust default permissions
        if OpenProcessToken(
            process_handle,
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY | TOKEN_ADJUST_DEFAULT,
            &mut token_handle,
        ) != 0 {
            // 1. Adjust token privileges to disable all
            AdjustTokenPrivileges(
                token_handle,
                1, // DisableAllPrivileges = TRUE (non-zero)
                std::ptr::null(), // NewState (ignored)
                0, // BufferLength
                std::ptr::null_mut(), // PreviousState
                std::ptr::null_mut(), // ReturnLength
            );
            
            // 2. Set Integrity Level to Low
            let mut low_integrity_sid = std::ptr::null_mut();
            let mut authority = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 16] }; // SECURITY_MANDATORY_LABEL_AUTHORITY
            
            if AllocateAndInitializeSid(
                &authority,
                1, // SubAuthorityCount
                0x1000, // SubAuthority0: SECURITY_MANDATORY_LOW_RID = 4096 (0x1000)
                0, 0, 0, 0, 0, 0, 0,
                &mut low_integrity_sid,
            ) != 0 {
                let mut label = TOKEN_MANDATORY_LABEL {
                    Label: SID_AND_ATTRIBUTES {
                        Sid: low_integrity_sid,
                        Attributes: SE_GROUP_INTEGRITY,
                    },
                };
                
                let success = SetTokenInformation(
                    token_handle,
                    TokenIntegrityLevel,
                    &label as *const _ as *const _,
                    std::mem::size_of::<TOKEN_MANDATORY_LABEL>() as u32,
                );
                
                FreeSid(low_integrity_sid);
                CloseHandle(token_handle);
                
                if success != 0 {
                    crate::log("[security] Windows Sandbox Active: Token privileges disabled, Integrity Level set to Low.");
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("SetTokenInformation failed"))
                }
            } else {
                CloseHandle(token_handle);
                Err(anyhow::anyhow!("AllocateAndInitializeSid failed"))
            }
        } else {
            Err(anyhow::anyhow!("OpenProcessToken failed"))
        }
    }
}

#[cfg(target_os = "macos")]
fn setup_macos_sandbox() -> Result<()> {
    use std::ffi::CString;
    use std::ptr;
    use libc::{c_char, c_int};

    extern "C" {
        fn sandbox_init(
            profile: *const c_char,
            flags: u64,
            errorbuf: *mut *mut c_char,
        ) -> c_int;
        fn sandbox_free_error(errorbuf: *mut c_char);
    }

    println!("[rjust] Setting up Seatbelt sandbox (macOS)");

    // Profile to deny all network activity and protect system resources
    // Since kSBXProfileNoNetwork is deprecated/private, we pass a custom SBPL profile that allows reading/writing 
    // to current directory and prevents networking.
    let profile_str = "
(version 1)
(deny default)
(allow file-read* file-write* (subpath \".\"))
(allow file-read* (subpath \"/usr/lib\"))
(allow file-read* (subpath \"/lib\"))
(allow file-read* (subpath \"/System/Library\"))
(allow mach-lookup)
(allow sysctl-read)
(allow process-exec (subpath \".\"))
";
    let profile = CString::new(profile_str).unwrap();
    let mut error_ptr: *mut c_char = ptr::null_mut();

    unsafe {
        let ret = sandbox_init(profile.as_ptr(), 0, &mut error_ptr);
        if ret != 0 {
            let error_msg = if !error_ptr.is_null() {
                let msg = std::ffi::CStr::from_ptr(error_ptr).to_string_lossy().into_owned();
                sandbox_free_error(error_ptr);
                msg
            } else {
                "Unknown macOS Seatbelt sandbox error".to_string()
            };
            return Err(anyhow::anyhow!("macOS Seatbelt initialization failed: {}", error_msg));
        }
    }

    crate::log("[security] macOS Seatbelt Sandbox Active: networking denied, filesystem restricted to workspace.");
    Ok(())
}


