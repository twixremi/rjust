use rjust::{security, loader, shm};
use shared_memory::ShmemConf;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Basic log redirection for worker
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("./rjust/worker.log") 
    {
        // Simple redirection using internal println! or similar would go here
    }
    
    println!("[worker] Starting rjust_worker process...");

    // 1. Open Shared Memory created by the Agent
    let shm = ShmemConf::new()
        .os_id(shm::SHM_LINK_NAME)
        .open()
        .map_err(|e| anyhow::anyhow!("Failed to open SHM: {}", e))?;

    let shm_ptr = shm.as_ptr() as *mut shm::SharedData;

    // 2. Initialize Security Sandbox (Isolated to this process!)
    if let Err(e) = security::initialize() {
        eprintln!("[worker] FAILED to initialize security: {}", e);
    }

    // 3. Load modules
    if let Err(e) = loader::load_all() {
        eprintln!("[worker] Error loading modules: {}", e);
    }

    println!("[worker] Initialization complete. Entering sync loop.");

    // 4. Main Loop: Modules will interact with SHM here
    loop {
        unsafe {
            let _ = (*shm_ptr).lock(|data| {
                // Here modules could read player_x/y/z and set command_pending
                // For demonstration, let's just log if a command was processed by agent
                if !data.command_pending && data.tick_counter % 100 == 0 {
                    // Example: Worker decides to send a chat message
                    /*
                    let msg = b"Hello from isolated worker!";
                    data.command_type = 1; // Chat
                    data.command_len = msg.len(); // FIX: Set the length!
                    data.command_payload[..msg.len()].copy_from_slice(msg);
                    data.command_pending = true;
                    */
                }
            });
        }
        
        thread::sleep(Duration::from_millis(10));
    }
}
