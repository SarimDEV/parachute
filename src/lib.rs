mod ffprobe;
mod manifest;
mod processing;
use dashmap::DashMap;
use processing::Video;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use std::thread;

use crate::ffprobe::{get_media_info, MediaInfo};
use crate::manifest::{create_manifest_file, create_manifest_file_subs, create_master_file};
use crate::processing::{process_video, Audio, Profile};

#[derive(Clone, Debug)]
pub enum ProcessingState {
    InProgress,
    Done,
}

pub struct Parachute {
    id_to_process: Arc<DashMap<String, ProcessingState>>,
    ffprobe_path: String,
    ffmpeg_path: String,
    cdn_dir_path: String,
}

impl Parachute {
    pub fn new(ffprobe_path: &str, ffmpeg_path: &str, cdn_dir_path: &str) -> Self {
        Self {
            id_to_process: Arc::new(DashMap::new()),
            ffprobe_path: ffprobe_path.to_string(),
            ffmpeg_path: ffmpeg_path.to_string(),
            cdn_dir_path: cdn_dir_path.to_string(),
        }
    }

    pub fn play_video_seq(&self, id: &str, path: &PathBuf) -> ProcessingState {
        self.id_to_process
            .entry(id.to_string())
            .or_insert_with(|| {
                let id_to_process = self.id_to_process.clone();
                let id = id.to_string();
                let mut process = self.start_process(&id, &path);
                thread::spawn(move || {
                    let _ = process.wait();
                    id_to_process.insert(id, ProcessingState::Done);
                });
                ProcessingState::InProgress
            })
            .clone()
    }

    fn start_process(&self, id: &str, path: &PathBuf) -> Child {
        let media_info = get_media_info(&self.ffprobe_path, path).unwrap();
        let video_args = Video::get_args(&media_info);
        let audio_args = Audio::get_args(&media_info);
        self.create_files(id, &media_info);
        process_video(
            path,
            id,
            &self.cdn_dir_path,
            &self.ffmpeg_path,
            &video_args,
            &audio_args,
        )
    }

    fn create_files(&self, id: &str, media_info: &MediaInfo) {
        let manifest_path = PathBuf::from(format!("{}/{}_manifest.m3u8", self.cdn_dir_path, id));
        let manifest_subs_path =
            PathBuf::from(format!("{}/{}_manifest_subs.m3u8", self.cdn_dir_path, id));
        let playlist_path = PathBuf::from(format!("{}/{}_playlist.m3u8", self.cdn_dir_path, id));
        let init_seg = format!("{}_init.mp4", id);
        let duration: f32 = media_info.format.duration.parse().unwrap();

        create_manifest_file(id, &manifest_path, duration, &init_seg).unwrap();
        create_manifest_file_subs(id, &manifest_subs_path, duration).unwrap();
        create_master_file(&manifest_path, &manifest_subs_path, &playlist_path).unwrap();
    }
}

