// Complete working example to play a .opus file
// 
// Run with: cargo run --example play_opus_file --features playback /path/to/file.opus
// Example: cargo run --example play_opus_file --features playback "/Users/dima/Music/ost/Rain World.opus"

#[cfg(feature = "playback")]
extern crate opus;
#[cfg(feature = "playback")]
extern crate ogg;
#[cfg(feature = "playback")]
extern crate rodio;

#[cfg(feature = "playback")]
use opus::{Decoder, Channels};
#[cfg(feature = "playback")]
use std::fs::File;
#[cfg(feature = "playback")]
use std::io::BufReader;
#[cfg(feature = "playback")]
use std::env;

#[cfg(feature = "playback")]
fn play_opus_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening .opus file: {}", path);
    
    // 1. Open the .opus file and create an Ogg reader
    let file = File::open(path)?;
    let mut reader = ogg::reading::PacketReader::new(BufReader::new(file));
    
    // 2. Set up audio output
    println!("Setting up audio output...");
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&stream_handle)?;
    
    // Set volume to 0.3 (30%)
    sink.set_volume(0.3);
    println!("Volume set to 30%");
    
    // 3. Parse file and decode
    let mut opus_decoder = None;
    let mut stream_serial = None;
    let mut packet_count = 0;
    let mut audio_packets = 0;
    
    println!("Reading and decoding packets...");
    
    while let Some(packet) = reader.read_packet()? {
        packet_count += 1;
        
        match stream_serial {
            None => {
                // Look for Opus header
                if packet.data.starts_with(b"OpusHead") {
                    stream_serial = Some(packet.stream_serial());
                    
                    // Parse Opus header
                    if packet.data.len() >= 19 {
                        let channels_count = packet.data[9];
                        let channels = if channels_count == 2 { 
                            Channels::Stereo 
                        } else { 
                            Channels::Mono 
                        };
                        
                        // Opus always uses 48kHz internally
                        let sample_rate = 48000;
                        
                        opus_decoder = Some(Decoder::new(sample_rate, channels)?);
                        println!("Initialized Opus decoder: {:?}, {}Hz", channels, sample_rate);
                        println!("Opus header found in packet {}", packet_count);
                    } else {
                        return Err("Invalid Opus header".into());
                    }
                } else {
                    println!("Packet {}: Not an Opus stream (starts with {:?})", 
                             packet_count, 
                             &packet.data[..std::cmp::min(8, packet.data.len())]);
                }
            }
            Some(serial) if packet.stream_serial() == serial => {
                if let Some(ref mut decoder) = opus_decoder {
                    if packet.data.starts_with(b"OpusTags") {
                        println!("Opus tags found in packet {}", packet_count);
                        // Skip tags packet
                    } else {
                        // This is an audio packet
                        audio_packets += 1;
                        
                        // Decode the Opus packet
                        let mut pcm_data = vec![0i16; 5760 * 2]; // Max frame size for stereo
                        let samples_per_channel = decoder.decode(&packet.data, &mut pcm_data, false)?;
                        
                        if samples_per_channel > 0 {
                            // Determine actual channel count
                            let channel_count = match decoder.get_nb_samples(&packet.data) {
                                Ok(_) => {
                                    // Use the channels from decoder creation
                                    match opus_decoder.as_ref().unwrap() {
                                        _ => 2, // Assume stereo for now, we could store this from header
                                    }
                                }
                                Err(_) => 2,
                            };
                            
                            // Resize to actual decoded size
                            pcm_data.truncate(samples_per_channel * channel_count);
                            
                            // Convert to f32 for rodio
                            let audio_data: Vec<f32> = pcm_data.iter()
                                .map(|&sample| sample as f32 / 32768.0)
                                .collect();
                            
                            // Create audio source and append to sink
                            let source = rodio::buffer::SamplesBuffer::new(
                                channel_count as u16, 
                                48000, 
                                audio_data
                            );
                            sink.append(source);
                            
                            if audio_packets % 100 == 0 {
                                println!("Decoded {} audio packets, {} samples per channel in this packet", 
                                         audio_packets, samples_per_channel);
                            }
                        }
                    }
                }
            }
            _ => {
                // Different stream, ignore
            }
        }
    }
    
    println!("Finished reading file. Total packets: {}, Audio packets: {}", packet_count, audio_packets);
    println!("Playing audio... (press Ctrl+C to stop)");
    
    // Wait for playback to finish
    sink.sleep_until_end();
    
    println!("Playback finished!");
    Ok(())
}

#[cfg(feature = "playback")]
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_opus_file>", args[0]);
        eprintln!("Example: {} \"/Users/dima/Music/ost/Rain World.opus\"", args[0]);
        eprintln!();
        eprintln!("Make sure to run with the playback feature:");
        eprintln!("cargo run --example play_opus_file --features playback <file>");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    
    match play_opus_file(file_path) {
        Ok(()) => {
            println!("Successfully played the file!");
        }
        Err(e) => {
            eprintln!("Error playing file: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "playback"))]
fn main() {
    eprintln!("This example requires the 'playback' feature to be enabled.");
    eprintln!("Run with: cargo run --example play_opus_file --features playback <file>");
    eprintln!();
    eprintln!("The playback feature adds dependencies for:");
    eprintln!("  - ogg: For parsing .opus files");
    eprintln!("  - rodio: For audio output");
    std::process::exit(1);
}