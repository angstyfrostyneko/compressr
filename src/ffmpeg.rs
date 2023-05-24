use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};

fn prettify_output(line: String, frame_count: u64) {
    println!("{}", line);
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
                input_file
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
// keep count of retries for prettify
pub fn encode(
    input_file: &str,
    output_file: &str,
    video_codec: &str,
    audio_codec: &str,
    video_bitrate: String,
    audio_bitrate: String,
) -> Result<(), Error> {
    let frame_count = get_frame_count(input_file);
    // todo: this function doesn't need to return anything
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
            "pipe:2",
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
            }
        ])
        .stderr(Stdio::piped())
        .spawn()?
        .stderr
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture error output."))?;
    let buffer = BufReader::new(stdout);
    buffer
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| prettify_output(line, frame_count));

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
            "pipe:2",
            "-pass",
            "2",
            "-c:a",
            audio_codec,
            "-b:a",
            &audio_bitrate,
            output_file,
        ])
        .stderr(Stdio::piped())
        .spawn()?
        .stderr
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture error output."))?;
    let buffer = BufReader::new(stdout);
    buffer
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| prettify_output(line, frame_count));

    Ok(())
}
