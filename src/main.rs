use confy;
mod ffmpeg;
use ffmpeg::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::metadata;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Config {
    gpu: String,
    codec: String,
    size_mb: f32,
    audio_bitrate: f32,
    delete_original: bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            gpu: String::from("none"),
            codec: String::from("h264"),
            size_mb: 25.0,
            audio_bitrate: 240.0,
            delete_original: false,
        }
    }
}
fn main() {
    let cli_args: Vec<String> = env::args().collect();
    let cfg: Config = confy::load("compressr", None).expect("Cannot load config file.");

    let input_file = Path::new(&cli_args[1]);
    let mut audio_codec = "aac";
    let mut video_extension = "mp4";

    let binding = cfg.codec.to_lowercase();
    let codec = binding.as_str();
    let binding = cfg.gpu.to_lowercase();
    let gpu = binding.as_str();

    // this entire part is really ugly, idk how to make it better
    let mut video_bitrate = cfg.size_mb * 8192.0 / get_duration(input_file) - cfg.audio_bitrate;
    // multiplying to go megabyte -> kilobit ^^^^^^^^
    let mut audio_bitrate = cfg.audio_bitrate;
    if video_bitrate < 0.0 {
        audio_bitrate += -video_bitrate;
        video_bitrate = cfg.size_mb * 8192.0 / get_duration(input_file);
    }

    let video_codec = match codec {
        "av1" => {
            video_extension = "mkv";
            match gpu {
                "amd" => "av1_amf",
                "intel" => "av1_qsv",
                "nvidia" => "av1_nvenc",
                _ => "libaom",
            }
        }
        "h265" => match gpu {
            "amd" => "hevc_amf",
            "intel" => "hevc_qsv",
            "nvidia" => "hevc_nvenc",
            _ => "libx265",
        },
        "vp9" => {
            video_extension = "webm";
            audio_codec = "libopus";
            match gpu {
                "intel" => "vp9_qsv",
                _ => "libvpx-vp9",
            }
        }
        _ => match gpu {
            "amd" => "h264_amf",
            "intel" => "h264_qsv",
            "nvidia" => "h264_nvenc",
            _ => "libx264",
        },
    };

    let output = input_file
        .to_owned()
        .with_file_name(format!(
            "{}mb {}",
            cfg.size_mb,
            &input_file.file_stem().unwrap().to_string_lossy()
        ))
        .with_extension(video_extension);

    loop {
        encode(
            &input_file.display().to_string(),
            &output.display().to_string(),
            video_codec,
            audio_codec,
            (video_bitrate * 1024.0).to_string(),
            (audio_bitrate * 1024.0).to_string(),
        )
        .unwrap();
        // 2-pass encoding isn't perfect, if it overshoots the filesize we try again with 95% of the bitrate
        video_bitrate *= 0.95;
        if (metadata(&output).unwrap().len() as f64) < (cfg.size_mb * 1000000.0).into() {
            break;
        }
    }
}
