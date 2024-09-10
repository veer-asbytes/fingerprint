use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use video_rs::{io::Reader, DecoderBuilder, Frame};

pub fn extract_frames10(video_path: &str) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
	// Initialize video_rs
	initialize_video_rs()?;

	// Open the video file using a buffered reader
	let video_file = File::open(video_path)?;
	let mut reader = BufReader::new(video_file);

	// Create a decoder for the video
	let mut decoder = DecoderBuilder::new().build_from_reader(&mut reader)?;

	let mut frames = Vec::new();
	let mut frame_count = 0;
	let segment_duration: i64 = 120;
	let mut segment_start_time = 0;

	while let Some(frame) = decoder.decode_next_frame()? {
		let current_frame_time = frame.timestamp().unwrap_or(0);
		if current_frame_time >= segment_start_time + segment_duration {
			let frame_data = frame.data(0).to_vec();
			frames.push(frame_data.clone()); // Clone `frame_data` before pushing
			segment_start_time = current_frame_time;
			frame_count += 1;
			eprintln!(
				"Extracted frame {} at time {} with size {}",
				frame_count,
				current_frame_time,
				frame_data.len()
			);
		}
	}

	eprintln!("Extracted {} frames from the video.", frame_count);

	Ok(frames)
}

pub fn compare_videos10(video_path1: &str, video_path2: &str) -> Result<f64, Box<dyn Error>> {
	// Extract frames from both videos using video_rs
	let frames1 = extract_frames(video_path1)?;
	let frames2 = extract_frames(video_path2)?;

	// Generate fingerprints for the frames
	let fingerprints1: HashSet<_> = generate_fingerprints(frames1).into_iter().collect();
	let fingerprints2: HashSet<_> = generate_fingerprints(frames2).into_iter().collect();

	// Calculate the similarity using the Jaccard index
	let intersection_size = fingerprints1.intersection(&fingerprints2).count();
	let union_size = fingerprints1.union(&fingerprints2).count();

	let similarity = if union_size == 0 {
		0.0
	} else {
		intersection_size as f64 / union_size as f64
	};

	Ok(similarity)
}
