use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};

fn prettify_output(buffer: BufReader<std::process::ChildStdout>) {
    buffer
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));
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
            "-pass",
            "1",
            "-fps_mode",
            "vfr",
            "-f",
            "null",
            "NUL",
        ])
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;
    let buffer = BufReader::new(stdout);
    prettify_output(buffer);

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
            "-pass",
            "2",
            "-c:a",
            audio_codec,
            "-b:a",
            &audio_bitrate,
            output_file,
        ])
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;
    let buffer = BufReader::new(stdout);
    prettify_output(buffer);

    Ok(())
}
