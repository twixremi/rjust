use ctor::ctor;
use std::sync::Once;
use jni::sys::{jint, JNI_OK};
use std::fs::{self, OpenOptions};
use std::process::Command;
use shared_memory::ShmemConf;
use std::path::Path;

pub mod security;
pub mod jni_bridge;
pub mod jvmti;
pub mod console;
pub mod input;
pub mod loader;
pub mod minecraft;
pub mod shm;

static INIT: Once = Once::new();

struct ShmPtr(*mut shm::SharedData);
unsafe impl Send for ShmPtr {}
unsafe impl Sync for ShmPtr {}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn platform_init_logs() {
    use std::os::unix::io::AsRawFd;
    
    // Ensure rjust directory exists in the root
    let _ = fs::create_dir_all("./rjust");

    // Redirect logs to the local rjust folder instead of /tmp
    if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./rjust/agent.log") 
    {
        let fd = file.as_raw_fd();
        unsafe {
            libc::dup2(fd, libc::STDOUT_FILENO);
            libc::dup2(fd, libc::STDERR_FILENO);
        }
    }
}

#[cfg(target_os = "windows")]
fn platform_init_logs() {
    let _ = fs::create_dir_all("./rjust");
}

pub fn log(msg: &str) {
    println!("[rjust] {}", msg);
}

pub fn initialize_after_vm_init() {
    log("--- Phase 2: Post-VMInit Initialization ---");
    
    // 1. Setup Shared Memory
    let shm = match ShmemConf::new()
        .os_id(shm::SHM_LINK_NAME)
        .size(shm::SHM_SIZE)
        .create() {
            Ok(m) => m,
            Err(_) => ShmemConf::new().os_id(shm::SHM_LINK_NAME).open().expect("Failed to open SHM")
        };

    let raw_ptr = shm.as_ptr() as *mut shm::SharedData;
    unsafe { (*raw_ptr).init().expect("Failed to init SHM"); }
    let safe_ptr = ShmPtr(raw_ptr);

    // 2. Start Worker (must be placed in ./rjust/ beforehand or downloaded)
    log("Starting rjust-worker process...");
    let worker_path = "./rjust/rjust-worker";
    if Path::new(worker_path).exists() {
        let _worker = Command::new(worker_path)
            .spawn()
            .expect("Failed to start worker process");
    } else {
        log("CRITICAL: rjust-worker not found in ./rjust/ folder!");
    }

    // 3. Sync Loop
    std::thread::spawn(move || {
        let ptr = safe_ptr;
        loop {
            if let Ok(pos) = minecraft::Minecraft::get_player_pos() {
                unsafe {
                    let _ = (*ptr.0).lock(|data| {
                        data.player_x = pos.0;
                        data.player_y = pos.1;
                        data.player_z = pos.2;
                        data.tick_counter += 1;

                        if data.command_pending {
                            if data.command_type == 1 {
                                // FIX: Use command_len to avoid reading empty bytes/nulls
                                let len = data.command_len.min(512);
                                let msg = std::str::from_utf8(&data.command_payload[..len]).unwrap_or("error");
                                let _ = minecraft::Minecraft::send_chat_message(msg);
                            }
                            data.command_pending = false;
                        }
                    });
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    std::mem::forget(shm);
    log("--- Agent Bridge Active ---");
}

#[no_mangle]
pub extern "C" fn Agent_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _options: *mut libc::c_char,
    _reserved: *mut libc::c_void,
) -> jint {
    INIT.call_once(|| {
        log("--- Phase 1: Agent_OnLoad ---");
        jni_bridge::set_jvm_ptr(vm);
        
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(5));
            initialize_after_vm_init();
        });
    });
    JNI_OK
}

#[ctor]
fn ctor_init() {
    platform_init_logs();
    log("--- Library Loaded ---");
}
