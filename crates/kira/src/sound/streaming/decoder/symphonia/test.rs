use std::io::Cursor;

use crate::{frame::Frame, sound::streaming::decoder::Decoder};

use super::SymphoniaDecoder;

// Each tone is 0.4 seconds of two sine waves at 16000 frames per second, so
// that no frame is silent. They were made with ffmpeg:
//
// ffmpeg -f lavfi -i 'aevalsrc=0.1*sin(2*PI*440*t)+0.05*sin(2*PI*1330*t):s=16000:d=0.4'
// -ac 1 -fflags +bitexact -flags:a +bitexact -map_metadata -1
// -c:a aac -b:a 12k -movflags +faststart tone.m4a
//
// with `-c:a libmp3lame -b:a 16k` and tone.mp3 for the MP3 tone and
// `-c:a libvorbis -b:a 16k` and tone.ogg for the Ogg Vorbis tone.

/// The number of frames to compare after each seek.
const COMPARED_FRAMES: usize = 1024;

/// The largest difference between two samples that still counts as the same
/// sample. A frame decoded without the frames before it is off by 0.07 or more.
const TOLERANCE: f32 = 1e-5;

fn decoder(bytes: &'static [u8]) -> SymphoniaDecoder {
	SymphoniaDecoder::new(Box::new(Cursor::new(bytes))).unwrap()
}

/// Decodes the whole file from the start, without seeking.
fn decode_straight(bytes: &'static [u8]) -> Vec<Frame> {
	let mut decoder = decoder(bytes);
	let mut frames = vec![];
	while let Some((_, packet_frames)) = decoder.decode_packet().unwrap() {
		frames.extend(packet_frames);
	}
	frames
}

/// Tests that the frames decoded after a seek are the same as the frames at
/// the same indices when the file is decoded straight through.
fn test_seek(bytes: &'static [u8]) {
	let straight = decode_straight(bytes);
	let mut decoder = decoder(bytes);
	// seek from the end backwards so the decoder still holds the state of
	// frames far later in the file when a seek lands near the start
	for index in (0..straight.len() - COMPARED_FRAMES).rev().step_by(61) {
		let seeked_to = decoder.seek(index).unwrap();
		assert!(seeked_to <= index);
		let mut decoded = vec![];
		for _ in 0..64 {
			decoded.extend(decoder.decode().unwrap());
			if decoded.len() >= COMPARED_FRAMES {
				break;
			}
		}
		assert!(decoded.len() >= COMPARED_FRAMES);
		for (i, frame) in decoded.iter().take(COMPARED_FRAMES).enumerate() {
			let expected = straight[seeked_to + i];
			assert!(
				(frame.left - expected.left).abs() < TOLERANCE
					&& (frame.right - expected.right).abs() < TOLERANCE,
				"frame {} after seeking to {} is {:?}, but should be {:?}",
				seeked_to + i,
				index,
				frame,
				expected
			);
		}
	}
}

#[test]
#[cfg(all(feature = "aac", feature = "isomp4"))]
fn seek_aac() {
	test_seek(include_bytes!("tone.m4a"));
}

#[test]
#[cfg(feature = "mp3")]
fn seek_mp3() {
	test_seek(include_bytes!("tone.mp3"));
}

#[test]
#[cfg(all(feature = "ogg", feature = "vorbis"))]
fn seek_ogg_vorbis() {
	test_seek(include_bytes!("tone.ogg"));
}
