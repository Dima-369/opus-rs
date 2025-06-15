// Complete example of how to play a .opus file
// 
// This example shows the complete dependencies and code structure needed
// to play a .opus file. The opus-rs crate only handles the Opus decoding part.
//
// To run this example, you would need to add these dependencies to Cargo.toml:
//
// [dependencies]
// opus = "0.3"
// ogg = "0.8"
// rodio = "0.17"
// 
// Then run: cargo run --example opus_player_complete /path/to/your/file.opus

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_opus_file>", args[0]);
        eprintln!("Example: {} /Users/dima/Music/ost/Rain\\ World.opus", args[0]);
        process::exit(1);
    }
    
    let file_path = &args[1];
    
    println!("This example demonstrates what you need to play: {}", file_path);
    println!();
    println!("IMPORTANT: This opus-rs crate only provides Opus packet encoding/decoding.");
    println!("To actually play .opus files, you need additional dependencies:");
    println!();
    println!("1. Add to your Cargo.toml:");
    println!("   [dependencies]");
    println!("   opus = \"0.3\"        # This crate - for Opus packet decoding");
    println!("   ogg = \"0.8\"         # For parsing .opus files (Ogg container)");
    println!("   rodio = \"0.17\"      # For audio playback (or use cpal)");
    println!();
    println!("2. Example code structure:");
    println!();
    
    print_example_code();
    
    // Demonstrate the opus decoder part that this crate provides
    demonstrate_opus_decoder();
}

fn print_example_code() {
    println!(r#"
use opus::{{Decoder, Channels}};
use std::fs::File;
use std::io::BufReader;

fn play_opus_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {{
    // 1. Open the .opus file and create an Ogg reader
    let file = File::open(path)?;
    let mut reader = ogg::reading::PacketReader::new(BufReader::new(file));
    
    // 2. Parse the Opus header to get audio parameters
    let mut opus_decoder = None;
    let mut stream_serial = None;
    
    // 3. Set up audio output
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&stream_handle)?;
    
    // 4. Read and process packets
    while let Some(packet) = reader.read_packet()? {{
        match stream_serial {{
            None => {{
                // First packet should be the Opus header
                if packet.data.starts_with(b"OpusHead") {{
                    stream_serial = Some(packet.stream_serial());
                    
                    // Parse Opus header (simplified)
                    let channels = if packet.data[9] == 2 {{ 
                        Channels::Stereo 
                    }} else {{ 
                        Channels::Mono 
                    }};
                    let sample_rate = 48000; // Opus always uses 48kHz internally
                    
                    opus_decoder = Some(Decoder::new(sample_rate, channels)?);
                    println!("Initialized Opus decoder: {{:?}}, 48kHz", channels);
                }}
            }}
            Some(serial) if packet.stream_serial() == serial => {{
                if let Some(ref mut decoder) = opus_decoder {{
                    if !packet.data.starts_with(b"OpusTags") {{
                        // Decode the Opus packet
                        let mut pcm_data = vec![0i16; 5760 * 2]; // Max frame size
                        let samples = decoder.decode(&packet.data, &mut pcm_data, false)?;
                        
                        // Resize to actual decoded size
                        pcm_data.truncate(samples * 2); // 2 channels
                        
                        // Convert to f32 and play
                        let audio_data: Vec<f32> = pcm_data.iter()
                            .map(|&sample| sample as f32 / 32768.0)
                            .collect();
                        
                        let source = rodio::buffer::SamplesBuffer::new(2, 48000, audio_data);
                        sink.append(source);
                    }}
                }}
            }}
            _ => {{}} // Different stream, ignore
        }}
    }}
    
    // Wait for playback to finish
    sink.sleep_until_end();
    Ok(())
}}
"#);
}

fn demonstrate_opus_decoder() {
    println!("3. What this opus-rs crate provides (demonstration):");
    println!();
    
    // Create a decoder
    match opus::Decoder::new(48000, opus::Channels::Stereo) {
        Ok(mut decoder) => {
            println!("✓ Successfully created Opus decoder (48kHz, Stereo)");
            
            // Test with empty packet (simulates packet loss with FEC)
            let mut output = vec![0i16; 960 * 2]; // 20ms of stereo audio
            match decoder.decode(&[], &mut output, true) {
                Ok(samples) => {
                    println!("✓ Decoded {} samples per channel from empty packet (FEC)", samples);
                }
                Err(e) => {
                    println!("✗ Failed to decode: {}", e);
                }
            }
            
            // Show decoder capabilities
            println!("✓ Decoder supports:");
            println!("  - Sample rates: 8, 12, 16, 24, 48 kHz");
            println!("  - Channels: Mono, Stereo");
            println!("  - Forward Error Correction (FEC)");
            println!("  - Packet loss concealment");
        }
        Err(e) => {
            println!("✗ Failed to create decoder: {}", e);
        }
    }
    
    println!();
    println!("To actually play your .opus file, implement the code structure shown above!");
}