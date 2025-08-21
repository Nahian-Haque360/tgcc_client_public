use obfstr::obfstr;
use once_cell::sync::Lazy;
use std::{sync::{Arc, Mutex}};

pub static UPLOAD_FILE_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("<UPLOAD_FILE_URL_base64>").to_string()
});
pub static CHAT_ID: Lazy<String> = Lazy::new(|| obfstr!("<CHAT_ID>").to_string());
pub static OUTPUT_DIR: Lazy<String> = Lazy::new(|| obfstr!("output_dir").to_string());  // output_path

pub static UPDATE_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("<UPDATE_URL_base64>").to_string()
});
pub static SENDMSG_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("").to_string()
});
pub static CREATEFORUMTOPIC_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("<CREATEFORUMTOPIC_URL_base64>").to_string()
});
pub static MESSAGE_THREAD_ID: Lazy<Arc<Mutex<i64>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
pub static SEND_PHOTO_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("<SEND_PHOTO_URL_base64>").to_string()
});
pub static BOT_NAME: Lazy<String> = Lazy::new(|| obfstr!("<BOT_NAME>").to_string());
pub static GET_FILE_PATH_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("<GET_FILE_PATH_URL_base64>").to_string()
});
pub static DOWNLOAD_FILE_URL: Lazy<String> = Lazy::new(|| {
    obfstr!("<DOWNLOAD_FILE_URL_base64>").to_string()
});
pub static SLEEP_TIME: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| Arc::new(Mutex::new(5)));