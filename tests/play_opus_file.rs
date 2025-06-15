// Test file for playing a .opus file
// 
// This test demonstrates the Opus decoding capabilities.
// For complete .opus file playback, use the "playback" feature:
// cargo test --features playback
//
// The playback feature adds:
// 1. ogg crate - for parsing .opus files (Ogg container format)
// 2. rodio crate - for audio output
// 3. This opus library handles the Opus packet decoding

extern crate opus;

use opus::{Decoder, Channels};
use std::fs::File;
use std::io::Read;

#[test]
fn test_opus_decoder_basic() {
    // Create a decoder for 48kHz stereo audio
    let mut decoder = Decoder::new(48000, Channels::Stereo).unwrap();
    
    // This is a minimal example showing how to use the decoder
    // In a real application, you would:
    // 1. Parse the .opus file to extract Opus packets
    // 2. Decode each packet using this decoder
    // 3. Send the decoded PCM data to an audio output device
    
    // Example with empty packet (simulates packet loss)
    let mut output = vec![0i16; 960 * 2]; // 20ms of stereo audio at 48kHz
    let samples_decoded = decoder.decode(&[], &mut output, true).unwrap();
    println!("Decoded {} samples per channel from empty packet", samples_decoded);
    
    // The decoder can handle forward error correction (FEC)
    assert_eq!(samples_decoded, 960); // 20ms worth of samples per channel
}

// This function shows what a complete .opus file player would look like
// (but requires additional dependencies not included in this crate)
#[allow(dead_code)]
fn play_opus_file_example(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Open and parse the .opus file
    // You would need the `ogg` crate for this:
    // ```
    // let mut file = File::open(file_path)?;
    // let mut reader = ogg::reading::PacketReader::new(file);
    // ```
    
    // Step 2: Create an Opus decoder
    // You need to get the sample rate and channel count from the Opus header
    let sample_rate = 48000; // This should come from the Opus header
    let channels = Channels::Stereo; // This should come from the Opus header
    let mut decoder = Decoder::new(sample_rate, channels)?;
    
    // Step 3: Set up audio output
    // You would need an audio library like `cpal` or `rodio`:
    // ```
    // let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    // let sink = rodio::Sink::try_new(&stream_handle)?;
    // ```
    
    // Step 4: Decode and play loop
    // ```
    // while let Some(packet) = reader.read_packet()? {
    //     if packet.stream_serial() == opus_stream_id {
    //         let mut pcm_output = vec![0i16; max_frame_size * channels as usize];
    //         let samples = decoder.decode(&packet.data, &mut pcm_output, false)?;
    //         
    //         // Convert to the format expected by your audio library
    //         // and send to audio output
    //         sink.append(/* converted audio data */);
    //     }
    // }
    // ```
    
    println!("To actually play {}, you need additional dependencies:", file_path);
    println!("Add to Cargo.toml:");
    println!("[dependencies]");
    println!("opus = \"0.3\"");
    println!("ogg = \"0.8\"");
    println!("rodio = \"0.17\"  # or cpal = \"0.15\"");
    
    Ok(())
}

#[test]
fn demonstrate_opus_file_requirements() {
    let file_path = "/Users/dima/Music/ost/Rain World.opus";
    
    // Check if file exists
    if std::path::Path::new(file_path).exists() {
        println!("Found opus file: {}", file_path);
        
        // Try to read some bytes to show it's a real file
        if let Ok(mut file) = File::open(file_path) {
            let mut buffer = [0u8; 32];
            if let Ok(bytes_read) = file.read(&mut buffer) {
                println!("Read {} bytes from file", bytes_read);
                println!("First few bytes: {:?}", &buffer[..std::cmp::min(bytes_read, 8)]);
                
                // Check if it starts with "OggS" (Ogg container signature)
                if buffer.starts_with(b"OggS") {
                    println!("✓ File appears to be an Ogg container (correct for .opus files)");
                } else {
                    println!("⚠ File doesn't start with OggS - might not be a valid .opus file");
                }
            }
        }
    } else {
        println!("File not found: {}", file_path);
        println!("This test will still demonstrate the Opus decoder usage");
    }
    
    // Show how to use the decoder part
    play_opus_file_example(file_path).unwrap();
    
    // If playback features are enabled, test them
    #[cfg(feature = "playback")]
    test_with_playback_features(file_path);
}

#[cfg(feature = "playback")]
fn test_with_playback_features(file_path: &str) {
    println!();
    println!("🎵 Playback features are enabled! Testing ogg parsing...");
    
    if std::path::Path::new(file_path).exists() {
        match std::fs::File::open(file_path) {
            Ok(file) => {
                let mut reader = ogg::reading::PacketReader::new(std::io::BufReader::new(file));
                let mut packet_count = 0;
                let mut opus_header_found = false;
                
                // Read first few packets to verify it's a valid .opus file
                while let Ok(Some(packet)) = reader.read_packet() {
                    packet_count += 1;
                    
                    if packet.data.starts_with(b"OpusHead") {
                        opus_header_found = true;
                        println!("✓ Found Opus header in packet {}", packet_count);
                        
                        if packet.data.len() >= 19 {
                            let channels = packet.data[9];
                            let pre_skip = u16::from_le_bytes([packet.data[10], packet.data[11]]);
                            let sample_rate = u32::from_le_bytes([
                                packet.data[12], packet.data[13], 
                                packet.data[14], packet.data[15]
                            ]);
                            
                            println!("  Channels: {}", channels);
                            println!("  Pre-skip: {}", pre_skip);
                            println!("  Original sample rate: {} Hz", sample_rate);
                            println!("  (Opus always decodes to 48kHz internally)");
                        }
                        break;
                    } else if packet.data.starts_with(b"OpusTags") {
                        println!("✓ Found Opus tags in packet {}", packet_count);
                    }
                    
                    if packet_count >= 5 {
                        break; // Don't read the whole file in tests
                    }
                }
                
                if opus_header_found {
                    println!("✓ File appears to be a valid .opus file!");
                    println!("✓ You can now play it with:");
                    println!("  cargo run --example play_opus_file --features playback \"{}\"", file_path);
                } else {
                    println!("⚠ No Opus header found in first {} packets", packet_count);
                }
            }
            Err(e) => {
                println!("✗ Could not open file: {}", e);
            }
        }
    } else {
        println!("File not found, but playback features are working!");
    }
}

// Example of how you might structure a complete opus player
#[allow(dead_code)]
struct OpusPlayer {
    decoder: Decoder,
    sample_rate: u32,
    channels: Channels,
}

#[allow(dead_code)]
impl OpusPlayer {
    fn new(sample_rate: u32, channels: Channels) -> Result<Self, opus::Error> {
        let decoder = Decoder::new(sample_rate, channels)?;
        Ok(OpusPlayer {
            decoder,
            sample_rate,
            channels,
        })
    }
    
    fn decode_packet(&mut self, opus_packet: &[u8]) -> Result<Vec<i16>, opus::Error> {
        // Calculate maximum possible frame size (120ms at 48kHz)
        let max_frame_size = 120 * self.sample_rate as usize / 1000;
        let mut output = vec![0i16; max_frame_size * self.channels as usize];
        
        let samples_per_channel = self.decoder.decode(opus_packet, &mut output, false)?;
        
        // Resize to actual decoded size
        output.truncate(samples_per_channel * self.channels as usize);
        
        Ok(output)
    }
}

#[test]
fn test_opus_player_struct() {
    let mut player = OpusPlayer::new(48000, Channels::Stereo).unwrap();
    
    // Test with empty packet (packet loss simulation)
    let decoded = player.decode_packet(&[]).unwrap();
    println!("Decoded {} samples from empty packet", decoded.len());
    
    // In a real scenario, you would get opus_packet from parsing the .opus file
    // let decoded = player.decode_packet(&opus_packet).unwrap();
    // Then send `decoded` to your audio output system
}