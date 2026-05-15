#[no_mangle]
pub extern "C" fn rjust_module_init() {
    println!("[rjust] >>> HELLO FROM TEST_MOD.RJAR! <<<");
    println!("[rjust] >>> Module initialization successful. <<<");
}
