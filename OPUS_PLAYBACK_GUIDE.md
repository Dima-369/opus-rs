# How to Play .opus Files with opus-rs

This guide explains how to play `.opus` files using the `opus-rs` crate.

## Important Note

The `opus-rs` crate **only provides Opus packet encoding and decoding**. It does not handle:
- Reading `.opus` files (which are Ogg containers)
- Audio output/playback
- File format parsing

## What You Need

To play `.opus` files, you need three components:

1. **opus-rs** - For decoding Opus packets (this crate)
2. **ogg** - For parsing .opus files (Ogg container format)
3. **Audio output library** - For playing the decoded audio (rodio, cpal, etc.)

## Dependencies

### Option 1: Use the built-in playback feature

```toml
[dependencies]
opus = { version = "0.3", features = ["playback"] }
```

### Option 2: Add dependencies manually

```toml
[dependencies]
opus = "0.3"        # This crate - Opus packet decoding
ogg = "0.8"         # For parsing .opus files
rodio = "0.17"      # For audio playback
```

## Complete Example

```rust
use opus::{Decoder, Channels};
use std::fs::File;
use std::io::BufReader;

fn play_opus_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open the .opus file
    let file = File::open(path)?;
    let mut reader = ogg::reading::PacketReader::new(BufReader::new(file));
    
    // 2. Set up audio output
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&stream_handle)?;
    
    // 3. Parse file and decode
    let mut opus_decoder = None;
    let mut stream_serial = None;
    
    while let Some(packet) = reader.read_packet()? {
        match stream_serial {
            None => {
                // Parse Opus header
                if packet.data.starts_with(b"OpusHead") {
                    stream_serial = Some(packet.stream_serial());
                    
                    let channels = if packet.data[9] == 2 { 
                        Channels::Stereo 
                    } else { 
                        Channels::Mono 
                    };
                    
                    opus_decoder = Some(Decoder::new(48000, channels)?);
                }
            }
            Some(serial) if packet.stream_serial() == serial => {
                if let Some(ref mut decoder) = opus_decoder {
                    if !packet.data.starts_with(b"OpusTags") {
                        // Decode Opus packet
                        let mut pcm_data = vec![0i16; 5760 * 2];
                        let samples = decoder.decode(&packet.data, &mut pcm_data, false)?;
                        
                        pcm_data.truncate(samples * 2);
                        
                        // Convert to f32 and play
                        let audio_data: Vec<f32> = pcm_data.iter()
                            .map(|&sample| sample as f32 / 32768.0)
                            .collect();
                        
                        let source = rodio::buffer::SamplesBuffer::new(2, 48000, audio_data);
                        sink.append(source);
                    }
                }
            }
            _ => {} // Different stream
        }
    }
    
    sink.sleep_until_end();
    Ok(())
}

fn main() {
    if let Err(e) = play_opus_file("/Users/dima/Music/ost/Rain World.opus") {
        eprintln!("Error: {}", e);
    }
}
```

## Testing the Decoder

You can test just the Opus decoding part (what this crate provides) with:

```bash
cargo test --test play_opus_file
```

## Running the Examples

### Complete working player (with playback feature):
```bash
cargo run --example play_opus_file --features playback "/path/to/your/file.opus"
```

### Information and code examples:
```bash
cargo run --example opus_player_complete "/path/to/your/file.opus"
```

## What opus-rs Provides

- ✅ Opus packet encoding/decoding
- ✅ Support for mono and stereo
- ✅ Multiple sample rates (8, 12, 16, 24, 48 kHz)
- ✅ Forward Error Correction (FEC)
- ✅ Packet loss concealment

## What opus-rs Does NOT Provide

- ❌ .opus file parsing (use `ogg` crate)
- ❌ Audio output (use `rodio`, `cpal`, etc.)
- ❌ File I/O utilities
- ❌ Audio format conversion

## Alternative Audio Libraries

Instead of `rodio`, you can use:
- `cpal` - Lower-level audio I/O
- `portaudio` - Cross-platform audio I/O
- `alsa` - Linux-specific
- `coreaudio` - macOS-specific

The key is that you need to:
1. Parse the .opus file with `ogg`
2. Decode Opus packets with `opus-rs`
3. Output PCM audio with your chosen audio library