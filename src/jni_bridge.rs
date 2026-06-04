use jni::{JavaVM, JNIEnv};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::cell::RefCell;

struct SendRawPtr(*mut jni::sys::JavaVM);
unsafe impl Send for SendRawPtr {}
unsafe impl Sync for SendRawPtr {}

static JVM_PTR: Lazy<Mutex<Option<SendRawPtr>>> = Lazy::new(|| Mutex::new(None));

thread_local! {
    static ATTACHED_ENV: RefCell<Option<jni::AttachGuard<'static>>> = RefCell::new(None);
}

/// Stores the JavaVM instance globally.
pub fn set_jvm_ptr(vm: *mut jni::sys::JavaVM) {
    let mut global_ptr = JVM_PTR.lock().unwrap();
    *global_ptr = Some(SendRawPtr(vm));
}

/// Safely obtains a reference to JNIEnv for the current thread via a closure.
/// This is the ONLY way to access JNIEnv now, ensuring safety.
pub fn with_env<F, R>(f: F) -> anyhow::Result<R>
where
    F: FnOnce(&mut JNIEnv) -> R,
{
    let raw_ptr = {
        let guard = JVM_PTR.lock().unwrap();
        guard.as_ref()
            .map(|p| p.0)
            .ok_or_else(|| anyhow::anyhow!("JVM pointer not initialized"))?
    };

    let jvm = unsafe { JavaVM::from_raw(raw_ptr)? };
    
    // Check if already attached
    if let Ok(mut env) = jvm.get_env() {
        return Ok(f(&mut env));
    }

    // Attach if needed
    ATTACHED_ENV.with(|cell| {
        let mut guard_opt = cell.borrow_mut();
        if guard_opt.is_none() {
            let guard = jvm.attach_current_thread()?;
            let guard_static = unsafe { std::mem::transmute::<jni::AttachGuard<'_>, jni::AttachGuard<'static>>(guard) };
            *guard_opt = Some(guard_static);
        }
        
        let env = guard_opt.as_mut().unwrap();
        Ok(f(env))
    })
}

pub fn attach() -> anyhow::Result<()> {
    with_env(|_| ())
}
