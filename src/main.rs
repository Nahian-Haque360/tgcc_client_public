// #![windows_subsystem = "windows"]
use crate::api::*;
use std::{fs, path::Path};
use tgcc_client::*;
use tokio::spawn;

#[tokio::main]
async fn main() {
    let _ = topic_init().await;

    // 创建输出目录
    if !Path::new(OUTPUT_DIR.as_str()).exists() {
        fs::create_dir(OUTPUT_DIR.as_str()).expect("Failed to create output directory");
    }

    // let _ = model::screen_shot().await;
    let handle = spawn(command());
    let _ = handle.await;
}
