use std::io::Write;
use soundio::{Context, SoundIo, SoundIoDevice, SoundIoFormat};

fn record_audio(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new()?;
    ctx.connect()?;
    ctx.flush_events();

    let default_in_device = ctx.default_input_device().expect("No input device");
    let mut in_stream = default_in_device.create_instream()?;
    in_stream.open(SoundIoFormat::S16LE, 2, 44100.0)?;

    let mut out_file = std::fs::File::create(filename)?;

    let mut buffer = vec![0; in_stream.bytes_per_frame() as usize * in_stream.sample_rate() as usize];

    loop {
        match in_stream.begin_read(&mut buffer) {
            Ok(frame_count) => {
                if frame_count == 0 {
                    break;
                }
                out_file.write_all(&buffer[..frame_count as usize * in_stream.bytes_per_frame() as usize])?;
                in_stream.end_read();
            }
            Err(e) => {
                println!("Error while reading from input stream: {}", e);
                break;
            }
        }
    }

    Ok(())
}

