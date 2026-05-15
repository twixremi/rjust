use raw_sync::locks::{Mutex, LockInit};
use raw_sync::Timeout;

/// Shared memory structure for Agent <-> Worker communication.
#[repr(C)]
pub struct SharedData {
    pub mutex_data: [u8; 128], 
    
    pub player_x: f64,
    pub player_y: f64,
    pub player_z: f64,
    pub tick_counter: u64,

    pub command_pending: bool,
    pub command_type: i32, 
    pub command_len: usize, // FIX: Track the actual length of the payload
    pub command_payload: [u8; 512],
}

impl SharedData {
    pub unsafe fn init(&mut self) -> anyhow::Result<()> {
        self.player_x = 0.0;
        self.player_y = 0.0;
        self.player_z = 0.0;
        self.tick_counter = 0;
        self.command_pending = false;
        self.command_len = 0;
        
        let ptr = self.mutex_data.as_mut_ptr();
        Mutex::new(ptr, ptr).map_err(|_| anyhow::anyhow!("Failed to init mutex"))?;
        
        Ok(())
    }

    /// Safely lock the shared memory with a timeout to prevent deadlocks if a process crashes.
    pub unsafe fn lock<F, R>(&mut self, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut SharedData) -> R,
    {
        let ptr = self.mutex_data.as_mut_ptr();
        let (mutex, _) = Mutex::from_existing(ptr, ptr)
            .map_err(|_| anyhow::anyhow!("Failed to map existing mutex"))?;
        
        // FIX: Use a timeout (100ms) instead of infinite wait to prevent Deadlock 
        // if the worker crashes while holding the lock.
        let _guard = mutex.try_lock(Timeout::Val(std::time::Duration::from_millis(100)))
            .map_err(|_| anyhow::anyhow!("Lock timeout - potential deadlock detected!"))?;
            
        let result = f(self);
        Ok(result)
    }
}

pub const SHM_LINK_NAME: &str = "rjust_shm_v2"; // Increment version after struct change
pub const SHM_SIZE: usize = std::mem::size_of::<SharedData>();
