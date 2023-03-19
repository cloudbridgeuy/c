use log::info;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::process::{Command, Stdio};

#[derive(Deserialize, Serialize, Debug)]
struct TranscriptionResponse {
    text: String,
}

fn wait_for_keypress() -> io::Result<()> {
    let mut buffer = [0; 1];
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_exact(&mut buffer)?;
    Ok(())
}

pub fn whisper(api_key: String) -> std::io::Result<String> {
    info!("Spawning the rec command");
    let tmp = String::from_utf8(
        Command::new("mktemp")
            .output()
            .expect("Failed to create temp file")
            .stdout,
    )
    .expect("Failed to parse mktemp")
    .trim()
    .to_string()
        + ".wav";
    let mut rec = Command::new("rec")
        .arg("-c")
        .arg("1")
        .arg("-r")
        .arg("48000")
        .arg(&tmp)
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to spawn rec command");

    println!("\n----------------------------------------------");
    println!("Recording! Press any key to stop the recording");
    println!("[Ctrl+C] cancels the recording");
    println!("----------------------------------------------\n");
    wait_for_keypress().expect("Failed to wait for keypress");

    info!("Stopping the rec command");
    rec.kill().expect("Failed to kill rec");

    info!("Running ffmpeg");
    Command::new("ffmpeg")
        .arg("-i")
        .arg(&tmp)
        .arg("-acodec")
        .arg("libmp3lame")
        .arg("-y")
        .arg(crate::CONFIG_DIRECTORY_PATH.to_owned() + "/whisper.mp3")
        .output()
        .expect("Failed to execute ffmpeg command");

    info!("Removing the tmp file");
    Command::new("rm")
        .arg("-rf")
        .arg(&tmp)
        .output()
        .expect("Failed to execute rm command");

    info!("Getting audio transcription");

    let response = String::from_utf8(
        Command::new("curl")
            .arg("-sX")
            .arg("POST")
            .arg("https://api.openai.com/v1/audio/transcriptions")
            .arg("-H")
            .arg("Content-Type: multipart/form-data")
            .arg("-H")
            .arg(("Authorization: Bearer ").to_owned() + &api_key)
            .arg("--form")
            .arg(("file=@").to_owned() + crate::CONFIG_DIRECTORY_PATH + "/whisper.mp3")
            .arg("--form")
            .arg("model=whisper-1")
            .output()
            .expect("Failed to execute request to the Whisper API")
            .stdout,
    )
    .expect("Failed to parse reponse")
    .trim()
    .to_string();

    info!("Removing the whisper.mp3 file");
    Command::new("rm")
        .arg("-rf")
        .arg(crate::CONFIG_DIRECTORY_PATH.to_owned() + "/whisper.mp3")
        .output()
        .expect("Failed to execute rm command");

    let body: TranscriptionResponse = serde_json::from_str(&response)?;

    println!("\n----------------------------------------------\n\n");

    Ok(body.text)
}
