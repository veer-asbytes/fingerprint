use ffmpeg_next::{codec, format, frame, media, packet, Error};
use std::path::Path;
use vid_dup_finder_lib::{NormalizedTolerance, VideoHash};

use blake3::Hasher;
use std::collections::HashSet;

pub fn extract_frames(video_path: &str) -> Result<Vec<Vec<u8>>, Error> {
	// Initialize the FFmpeg library
	ffmpeg_next::init()?;

	// Open the video file
	let mut ictx = format::input(&video_path)?;

	// Find the video stream
	let input_stream_index = ictx
		.streams()
		.best(media::Type::Video)
		.ok_or(Error::StreamNotFound)?
		.index();

	// Get codec parameters from the stream
	let codec_params = ictx
		.stream(input_stream_index)
		.ok_or(Error::StreamNotFound)?
		.parameters();

	let codec = codec::Id::from(codec_params.id());
	let mut decoder = codec::Context::from_parameters(codec_params)?
		.decoder()
		.video()?;

	let mut frame = frame::Video::empty();
	let mut frames = Vec::new();

	let mut packet_count = 0;
	let mut frame_count = 0;
	let mut segment_start_time = 0;
	let segment_duration: i64 = 120;
	for (stream, packet) in ictx.packets() {
		packet_count += 1;
		if stream.index() == input_stream_index {
			decoder.send_packet(&packet)?;
			while let Ok(()) = decoder.receive_frame(&mut frame) {
				let current_frame_time = frame.timestamp().unwrap_or(0);
				if current_frame_time >= segment_start_time + segment_duration {
					let frame_data = frame.data(0).to_vec();
					frames.push(frame_data.clone()); // Clone `frame_data` before pushing
					segment_start_time = current_frame_time;
					frame_count += 1;
					eprintln!(
						"Extracted summary frame {} at time {} with size {}",
						frame_count,
						current_frame_time,
						frame_data.len()
					);
				}
			}
		}
	}
	eprintln!(
		"Processed {} packets and extracted {} frames",
		packet_count, frame_count
	);

	Ok(frames)
}

pub fn hash_frame(frame: &[u8]) -> Vec<u8> {
	println!("Hashing frame with size {}", frame.len());
	let mut hasher = Hasher::new();
	hasher.update(frame);
	let hash = hasher.finalize().as_bytes().to_vec();
	println!("Hash: {:?}", hash);
	hash
}

/// Generates a vector of fingerprints for the given video frames.
///
/// This function takes a vector of frames (each represented as a vector of bytes)
/// and generates a fingerprint for each frame using the SHA-256 hash function.
///
/// # Arguments
///
/// * `frames` - A vector of video frames, where each frame is a `Vec<u8>` representing the frame's raw data.
///
/// # Returns
///
/// * `Vec<Vec<u8>>` - A vector of fingerprints, where each fingerprint is a `Vec<u8>` representing the SHA-256 hash of the corresponding frame.
pub fn generate_fingerprints(frames: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
	frames
		.into_iter()
		.map(|frame| {
			let hash = hash_frame(&frame);
			println!("Frame hash: {:?}", hash);
			hash
		})
		.collect()
}

/// Compares two videos by extracting frames and generating fingerprints, then computing the similarity between the two sets of fingerprints.
///
/// This function extracts frames from the two provided video files, generates fingerprints for each frame,
/// and compares the fingerprints to determine the similarity between the two videos.
///
/// # Arguments
///
/// * `video_path1` - A string slice that holds the path to the first video file.
/// * `video_path2` - A string slice that holds the path to the second video file.
///
/// # Returns
///
/// * `Result<f64, Box<dyn std::error::Error>>` - The similarity score between the two videos as a floating-point value (0.0 to 1.0).
///    Returns an error if there is an issue with extracting frames or generating fingerprints.
///
/// # Errors
///
/// This function will return an error if:
/// * There is an issue with opening or reading the video files.
/// * There is an issue with extracting frames from the video files.
/// * There is an issue with generating fingerprints from the frames.
pub fn compare_videos(
	video_path1: &str,
	video_path2: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
	// Generate hashes for the videos using vid_dup_finder_lib
	let video_hash1 = VideoHash::from_path(video_path1)?;
	let video_hash2 = VideoHash::from_path(video_path2)?;

	// Use NormalizedTolerance to set the tolerance level for finding duplicates
	let tolerance = NormalizedTolerance::default();

	// Perform the search for duplicates using the generated hashes
	let dup_groups = vid_dup_finder_lib::search(vec![video_hash1, video_hash2], tolerance);

	// Calculate similarity score based on the duplicate groups
	let similarity = if dup_groups.len() == 1 {
		1.0 // Both videos are considered duplicates
	} else {
		0.0 // Videos are not considered duplicates
	};

	Ok(similarity)
}
/// Compares two videos to determine their similarity based on the intersection of their frame fingerprints.
///
/// This function extracts frames from the two given video files and generates fingerprints for each frame.
/// It then calculates the similarity between the two videos based on the Jaccard index of their frame fingerprints.
///
/// The similarity score is a floating-point value between 0.0 and 1.0, where 1.0 indicates identical videos
/// (i.e., all frame fingerprints match) and 0.0 indicates no similarity (i.e., no frame fingerprints match).
///
/// # Arguments
///
/// * `video_path1` - A string slice representing the path to the first video file.
/// * `video_path2` - A string slice representing the path to the second video file.
///
/// # Returns
///
/// Returns a `Result<f64, Box<dyn std::error::Error>>`:
/// * On success, returns the similarity score as a `f64`.
/// * On failure, returns an error wrapped in a `Box<dyn std::error::Error>`.
///
/// # Example
///
/// ```
/// let video_path1 = "path/to/first/video.mp4";
/// let video_path2 = "path/to/second/video.mp4";
///
/// match compare_videos5(video_path1, video_path2) {
///     Ok(similarity) => println!("The similarity between the videos is: {:.2}", similarity),
///     Err(e) => eprintln!("Error comparing videos: {}", e),
/// }
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// * There is an issue extracting frames from either video (e.g., file not found, format not supported).
/// * The fingerprint generation or similarity calculation encounters an error.

pub fn compare_videos5(
	video_path1: &str,
	video_path2: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
	let frames1 = extract_frames(video_path1)?;
	let frames2 = extract_frames(video_path2)?;

	let fingerprints1: HashSet<_> = generate_fingerprints(frames1).into_iter().collect();
	let fingerprints2: HashSet<_> = generate_fingerprints(frames2).into_iter().collect();

	let intersection_size = fingerprints1.intersection(&fingerprints2).count();
	let union_size = fingerprints1.union(&fingerprints2).count();

	let similarity = if union_size == 0 {
		0.0
	} else {
		intersection_size as f64 / union_size as f64
	};

	Ok(similarity)
}

// fn calculate_similarity(fingerprint1: &[u8], fingerprint2: &[u8]) -> f64 {
// 	// Implement a similarity calculation (e.g., Hamming distance, cosine similarity, etc.)
// 	// For simplicity, this example assumes a basic byte-wise comparison.
// 	let len = fingerprint1.len().min(fingerprint2.len());
// 	let mut match_count = 0;
//
// 	for i in 0..len {
// 		if fingerprint1[i] == fingerprint2[i] {
// 			match_count += 1;
// 		}
// 	}
//
// 	match_count as f64 / len as f64
// }
