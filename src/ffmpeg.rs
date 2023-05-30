use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};

fn prettify_output(
    buffer: BufReader<std::process::ChildStdout>,
    total_frames: u64,
    encode_counter: u8,
    current_pass: u8,
) {
    let mut fps = 0;
    let mut current_frame: u64 = 0;
    let encode_counter = encode_counter * 2;
    let current_pass = encode_counter - (1 - current_pass);

    buffer
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            if line.contains("frame") {
                current_frame = line.split('=').nth(1).unwrap().parse::<u64>().unwrap()
            }
            if line.contains("fps") {
                fps = line.split('=').nth(1).unwrap().parse::<f32>().unwrap() as i32
            }
            if fps != 0 && current_frame != 0 {
                let percentage = ((current_frame as f64 / total_frames as f64) * 100.0) as u64;
                let time_left = (total_frames - current_frame) / fps as u64;
                let progress: f32 = percentage as f32 / 100.0 * 16.0; // 16 characters for the entire progress bar
                let progress_bar = "█".repeat(progress as usize);
                let progress_bar_left = " ".repeat(16 - progress as usize);
                println!(
                    "fps {fps}         | frame {current_frame}/{total_frames}
{progress_bar}{progress_bar_left}| {percentage}%
pass            | {current_pass}/{encode_counter}
time left pass  | {time_left} seconds \x1b[4F" // going up 4 lines to write in place
                );
                fps = 0;
                current_frame = 0
            }
        });
}

pub fn get_frame_count(input_file: &str) -> u64 {
    let output = {
        Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-count_packets",
                "-show_entries",
                "stream=nb_read_packets",
                "-of",
                "csv=p=0",
                input_file,
            ])
            .output()
            .expect("failed to execute process")
    };
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse::<u64>()
        .unwrap()
}

pub fn get_duration(input_file: &Path) -> f32 {
    let output = {
        Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                input_file.to_str().unwrap(),
            ])
            .output()
            .expect("failed to execute process")
    };
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse::<f32>()
        .unwrap()
}
// todo: keep count of retries for prettify
#[allow(clippy::too_many_arguments)]
pub fn encode(
    input_file: &str,
    output_file: &str,
    video_codec: &str,
    audio_codec: &str,
    video_bitrate: String,
    audio_bitrate: String,
    encode_counter: u8,
) -> Result<(), Error> {
    let total_frames = get_frame_count(input_file);

    // pass 1
    let stdout = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input_file,
            "-c:v",
            video_codec,
            "-b:v",
            &video_bitrate,
            "-progress",
            "-",
            "-nostats",
            "-pass",
            "1",
            "-fps_mode",
            "vfr",
            "-f",
            {
                if cfg!(target_os = "windows") {
                    "null"
                } else {
                    "/dev/null"
                }
            },
            {
                if cfg!(target_os = "windows") {
                    "NUL"
                } else {
                    ""
                }
            },
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture error output."))?;
    let buffer = BufReader::new(stdout);
    prettify_output(buffer, total_frames, encode_counter, 0);

    // pass 2
    let stdout = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input_file,
            "-c:v",
            video_codec,
            "-b:v",
            &video_bitrate,
            "-progress",
            "-",
            "-nostats",
            "-pass",
            "2",
            "-c:a",
            audio_codec,
            "-b:a",
            &audio_bitrate,
            output_file,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture error output."))?;
    let buffer = BufReader::new(stdout);
    prettify_output(buffer, total_frames, encode_counter, 1);

    Ok(())
}
