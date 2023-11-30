use crate::ffprobe::MediaInfo;
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
};

pub struct Video;

pub trait Profile {
    fn transmux(stream_idx: u8) -> Vec<String>;
    fn transcode() -> Vec<String>;
    fn get_args(media_info: &MediaInfo) -> Vec<String>;
}

impl Profile for Video {
    fn transmux(stream_idx: u8) -> Vec<String> {
        println!("transmux video");

        [
            "-map",
            format!("0:v:{}", stream_idx).as_str(),
            "-c:v",
            "copy",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn transcode() -> Vec<String> {
        println!("transcode video");
        ["-map", "0:v:0", "-c:v", "libx264"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn get_args(media_info: &MediaInfo) -> Vec<String> {
        let video_streams = &media_info.video;

        for (i, video_stream) in video_streams.iter().enumerate() {
            if video_stream.codec_name == "h264" {
                return Self::transmux(i as u8);
            }
        }

        Self::transcode()
    }
}

pub struct Audio;

impl Profile for Audio {
    fn transmux(stream_idx: u8) -> Vec<String> {
        println!("transmux audio");
        [
            "-map",
            format!("0:a:{}", stream_idx).as_str(),
            "-c:a",
            "copy",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn transcode() -> Vec<String> {
        println!("transcode audio");
        ["-map", "0:a:0", "-c:a", "aac"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn get_args(media_info: &MediaInfo) -> Vec<String> {
        let audio_streams = &media_info.audio;

        for (i, audio_stream) in audio_streams.iter().enumerate() {
            if audio_stream.codec_name == "aac" {
                return Self::transmux(i as u8);
            }
        }

        Self::transcode()
    }
}

pub struct Subtitles;

impl Profile for Subtitles {
    fn transmux(stream_idx: u8) -> Vec<String> {
        println!("transmux subtitles");
        [
            "-map",
            format!("0:a:{}", stream_idx).as_str(),
            "-c:s",
            "copy",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn transcode() -> Vec<String> {
        println!("transcode subtitles");
        ["-map", "0:s:0", "-c:s", "webvtt"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn get_args(media_info: &MediaInfo) -> Vec<String> {
        let subtitle_streams = &media_info.subtitle;
        if subtitle_streams.len() == 0 {
            return Vec::new();
        }

        Self::transcode()
    }
}

pub fn process_video(
    input: &PathBuf,
    id: &str,
    cdn_dir_path: &str,
    ffmpeg_path: &str,
    video_args: &Vec<String>,
    audio_args: &Vec<String>,
    subtitle_args: &Vec<String>,
) -> Child {
    let media_file_path = input.to_string_lossy().to_string();
    let init_seg = format!("{id}_init.mp4");
    let segment_file_path = format!("{cdn_dir_path}/{id}_%d.m4s");
    let output_file_path = format!("{cdn_dir_path}/{id}.m3u8");

    let mut args = vec![
        "-v",
        "error",
        "-ss",
        "0",
        "-i",
        &media_file_path,
        "-copyts",
        "-y",
    ];

    let video_args: Vec<&str> = video_args.iter().map(|s| s.as_str()).collect();
    let audio_args: Vec<&str> = audio_args.iter().map(|s| s.as_str()).collect();
    let subtitle_args: Vec<&str> = subtitle_args.iter().map(|s| s.as_str()).collect();

    args.extend(&video_args);
    args.extend(&audio_args);
    args.extend(&subtitle_args);
    args.extend([
        "-start_at_zero".into(),
        "-vsync".into(),
        "passthrough".into(),
        "-avoid_negative_ts".into(),
        "disabled".into(),
        "-max_muxing_queue_size".into(),
        "2048".into(),
        "-f",
        "hls",
        "-start_number",
        "0",
        "-hls_flags".into(),
        "temp_file".into(),
        "-max_delay".into(),
        "5000000".into(),
        "-hls_fmp4_init_filename",
        &init_seg,
        "-hls_time",
        "10",
        "-force_key_frames",
        "expr:gte(t,n_forced*10)",
        "-hls_segment_type",
        "1",
        "-hls_segment_filename",
        &segment_file_path,
        &output_file_path,
    ]);

    // let args = vec![
    //     "-v",
    //     "error",
    //     "-ss",
    //     "0",
    //     "-i",
    //     &media_file_path,
    //     "-copyts".into(),
    //     "-y",
    //     "-map",
    //     "0:v:0",
    //     "-c:v",
    //     "copy",
    //     "-map",
    //     "0:a:0",
    //     "-c:a",
    //     "aac",
    //     "-ac",
    //     "2",
    //     "-map",
    //     "0:s:0",
    //     "-c:s",
    //     "webvtt",
    //     "-start_at_zero".into(),
    //     "-vsync".into(),
    //     "passthrough".into(),
    //     "-avoid_negative_ts".into(),
    //     "disabled".into(),
    //     "-max_muxing_queue_size".into(),
    //     "2048".into(),
    //     "-f",
    //     "hls",
    //     "-start_number",
    //     "0",
    //     "-hls_flags".into(),
    //     "temp_file".into(),
    //     "-max_delay".into(),
    //     "5000000".into(),
    //     "-hls_fmp4_init_filename",
    //     &init_seg,
    //     "-hls_time",
    //     "10",
    //     "-force_key_frames",
    //     "expr:gte(t,n_forced*10)",
    //     "-hls_segment_type",
    //     "1",
    //     "-hls_segment_filename",
    //     &segment_file_path,
    //     &output_file_path,
    // ];

    let child_process = Command::new(ffmpeg_path)
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run ffmpeg command");

    child_process
}

// pub fn transmux(input: &PathBuf, id: &str) -> Vec<String> {
//     let media_file_path = input.to_string_lossy().to_string();
//     println!("STARTING {} from {}", id, media_file_path);
//     let init_seg = format!("{id}_init.mp4");
//     let segment_file_path = format!("{CDN_DIR}/{id}_%d.m4s");
//     let output_file_path = format!("{CDN_DIR}/{id}.m3u8");
//     println!(
//         "{} {} {} {}",
//         media_file_path, init_seg, segment_file_path, output_file_path
//     );
//
//     let args = vec![
//         "-v",
//         "error",
//         "-ss",
//         "0",
//         "-i",
//         &media_file_path,
//         "-copyts".into(),
//         "-y",
//         "-map",
//         "0:v:0",
//         "-c:v",
//         "copy",
//         "-map",
//         "0:a:0",
//         "-c:a",
//         "aac",
//         "-ac",
//         "2",
//         "-map",
//         "0:s:0",
//         "-c:s",
//         "webvtt",
//         "-start_at_zero".into(),
//         "-vsync".into(),
//         "passthrough".into(),
//         "-avoid_negative_ts".into(),
//         "disabled".into(),
//         "-max_muxing_queue_size".into(),
//         "2048".into(),
//         "-f",
//         "hls",
//         "-start_number",
//         "0",
//         "-hls_flags".into(),
//         "temp_file".into(),
//         "-max_delay".into(),
//         "5000000".into(),
//         "-hls_fmp4_init_filename",
//         &init_seg,
//         "-hls_time",
//         "10",
//         "-force_key_frames",
//         "expr:gte(t,n_forced*10)",
//         "-hls_segment_type",
//         "1",
//         "-hls_segment_filename",
//         &segment_file_path,
//         &output_file_path,
//     ];
//
//     args.into_iter().map(String::from).collect()
//
//     // println!("Here!");
//     // let child_process = Command::new(FFMPEG_PATH)
//     //     .args(&args)
//     //     .stdout(Stdio::piped())
//     //     .spawn()
//     //     .expect("Failed to run ffmpeg command");
//     //
//     // println!("Here abc!");
//     // child_process
// }
