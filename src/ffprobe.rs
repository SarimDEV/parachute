use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf, process::Command};

#[derive(Serialize, Deserialize, Debug)]
pub struct FFPFormat {
    pub filename: String,
    pub nb_streams: u32,
    pub format_name: String,
    pub format_long_name: String,
    pub start_time: String,
    pub duration: String,
    pub size: String,
    pub bit_rate: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FFPStream {
    pub codec_name: String,
    pub codec_type: String,
    pub duration: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bit_rate: Option<String>,
    pub r_frame_rate: Option<String>,
}

#[derive(Debug)]
pub struct Video {
    pub codec_name: String,
    pub duration: f32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Audio {
    pub codec_name: String,
    pub duration: f32,
}

#[derive(Debug)]
pub struct Subtitle {
    pub codec_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FFprobe {
    pub format: FFPFormat,
    pub streams: Vec<FFPStream>,
}

pub fn ffprobe(ffprobe_path: &str, path: &PathBuf) -> Result<FFprobe, Box<dyn Error>> {
    let output = Command::new(ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-of")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(&path)
        .output()
        .expect("Failed to run ffprobe command");
    let output_str = String::from_utf8(output.stdout).unwrap();
    Ok(serde_json::from_str(&output_str)?)
}

#[derive(Debug)]
pub struct MediaInfo {
    pub format: FFPFormat,
    pub video: Vec<Video>,
    pub audio: Vec<Audio>,
    pub subtitle: Vec<Subtitle>,
}

pub fn get_media_info(ffprobe_path: &str, path: &PathBuf) -> Result<MediaInfo, Box<dyn Error>> {
    let metadata = ffprobe(ffprobe_path, path).expect("Unable to get media info");
    let mut videos: Vec<Video> = Vec::new();
    let mut audios: Vec<Audio> = Vec::new();
    let mut subtitles: Vec<Subtitle> = Vec::new();

    for stream in metadata.streams.into_iter() {
        match stream.codec_type.as_str() {
            "video" => {
                let video_duration = {
                    if let Some(duration) = stream.duration {
                        duration.parse::<f32>().unwrap()
                    } else {
                        metadata.format.duration.parse::<f32>().unwrap()
                    }
                };
                let video = Video {
                    codec_name: stream.codec_name,
                    duration: video_duration,
                    width: stream.width.expect("Video has no width specified"),
                    height: stream.height.expect("Video has no height specified"),
                };
                videos.push(video)
            }
            "audio" => {
                let audio_duration = {
                    if let Some(duration) = stream.duration {
                        duration.parse::<f32>().unwrap()
                    } else {
                        metadata.format.duration.parse::<f32>().unwrap()
                    }
                };

                let audio = Audio {
                    codec_name: stream.codec_name,
                    duration: audio_duration,
                };
                audios.push(audio)
            }
            "subtitle" => {
                let subtitle = Subtitle {
                    codec_name: stream.codec_name,
                };
                subtitles.push(subtitle)
            }
            _ => {}
        }
    }

    Ok(MediaInfo {
        format: metadata.format,
        video: videos,
        audio: audios,
        subtitle: subtitles,
    })
}
