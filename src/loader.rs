use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// A global list of loaded modules
static LOADED_MODULES: Lazy<Mutex<Vec<Library>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn load_all() -> Result<()> {
    // All files are now in the root rjust folder
    let mods_dir = Path::new("./rjust");
    
    if !mods_dir.exists() {
        println!("[rjust] Root directory not found, creating: {:?}", mods_dir);
        fs::create_dir_all(mods_dir)?;
        return Ok(());
    }

    println!("[rjust] Scanning for modules in {:?}", mods_dir);

    let mut found = 0;
    for entry in fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        
        if path.extension().map_or(false, |ext| ext == "rjar") {
            println!("[rjust] Found module: {}", file_name);
            found += 1;
            
            if let Err(e) = load_module_with_protection(&path) {
                eprintln!("\n[rjust] !!! WARNING !!!");
                eprintln!("[rjust] Module {:?} is incompatible or corrupted.", file_name);
                eprintln!("[rjust] Error: {}", e);
                eprintln!("[rjust] The game will continue to load, but this module will be inactive.\n");
            }
        }
    }

    if found == 0 {
        println!("[rjust] No .rjar modules found in the root directory.");
    } else {
        println!("[rjust] Successfully processed {} module(s).", found);
    }

    Ok(())
}

fn load_module_with_protection(path: &Path) -> Result<()> {
    let result = std::panic::catch_unwind(|| {
        load_module(path)
    });

    match result {
        Ok(res) => res,
        Err(_) => Err(anyhow::anyhow!("Module initialization panicked")),
    }
}

fn load_module(path: &Path) -> Result<()> {
    verify_signature(path).context("Signature verification failed")?;

    // Canonicalize path to ensure correct DLL load when privileges or directory resolution is restricted (e.g. Windows Low Integrity)
    let abs_path = fs::canonicalize(path).context("Failed to canonicalize library path")?;

    unsafe {
        let lib = Library::new(&abs_path)?;
        if let Ok(init_fn) = lib.get::<Symbol<unsafe extern "C" fn()>>(b"rjust_module_init") {
            println!("[rjust] Calling entry point for {:?}", abs_path.file_name().unwrap());
            init_fn();
        }
        let mut modules = LOADED_MODULES.lock().unwrap();
        modules.push(lib);
    }

    Ok(())
}

fn verify_signature(_path: &Path) -> Result<()> {
    Ok(())
}
