// use ez_ffmpeg::{FfmpegContext, Input, Output};
// fn screen_watch() -> Result<(), Box<dyn std::error::Error>> {
//     let input = Input::from("E:\\20250609_201226.mp4");
//     let output = Output::from("rtmps://dc5-1.rtmp.t.me/s/2623611285:StjrMdJa1hHQaSNSH35e4g");
//     FfmpegContext::builder()
//         .input(input)
//         .output(output)
//         .build()?
//         .start()?
//         .wait()?;
//     Ok(())
// }

// #[test]
// fn test_screen_watch() {
//     match screen_watch() {
//         Ok(_) => println!("Screen watch started successfully."),
//         Err(e) => eprintln!("Error starting screen watch: {}", e),
//     }
// }