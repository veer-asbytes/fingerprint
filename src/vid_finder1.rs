use std::collections::HashSet;
use std::error::Error;
use vid_dup_finder_lib::{NormalizedTolerance, VideoHash};
/// Extracts frames from a video and filters out similar frames based on frame hashing.
///
/// This function extracts frames from the given video and calculates a hash for each frame. It then
/// filters out frames that have duplicate hashes based on a given tolerance level.
///
/// # Arguments
///
/// * `video_path` - The path to the video file from which frames will be extracted.
/// * `tolerance` - The tolerance level for filtering out similar frames (not used in the current implementation but provided for reference).
///
/// # Returns
///
/// This function returns a `Result` containing a vector of unique frames (each frame represented as a `Vec<u8>`) if successful.
/// If any error occurs during frame extraction or processing, it returns an error.
///
/// # Errors
///
/// This function will return an error if frame extraction fails or if any other error occurs during processing.

pub fn extract_and_filter_frames1(
	video_path: &str,
	tolerance: f64,
) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
	// Extract frames from the video
	let frames = extract_frames(video_path)?;

	// Create hashes for each frame
	let hashes: Vec<(Vec<u8>, Vec<u8>)> = frames
		.iter()
		.map(|frame| {
			let hash = hash_frame(frame);
			(frame.clone(), hash)
		})
		.collect();

	// Initialize tolerance (this may not be used directly here, but for reference)
	let normalized_tolerance = NormalizedTolerance::new(tolerance);

	// Use vid_dup_finder_lib to find duplicate hashes within the same video
	let mut unique_frames = Vec::new();
	let mut seen_hashes = HashSet::new();

	for (frame, hash) in hashes {
		let hash_str = hex::encode(&hash);
		if !seen_hashes.contains(&hash_str) {
			seen_hashes.insert(hash_str);
			unique_frames.push(frame);
		}
	}

	Ok(unique_frames)
}
/// Compares two videos for similarity based on frame hashes.
///
/// This function extracts and filters frames from both videos, generates hashes for the entire videos,
/// and then compares the video hashes to determine similarity. The similarity score is based on the
/// number of duplicate groups found between the two videos.
///
/// # Arguments
///
/// * `video_path1` - The path to the first video file to be compared.
/// * `video_path2` - The path to the second video file to be compared.
///
/// # Returns
///
/// This function returns a `Result` containing a similarity score between 0.0 and 1.0. A score of 1.0
/// indicates that the videos are considered similar (duplicates), while a score of 0.0 indicates no similarity.
///
/// # Errors
///
/// This function will return an error if any of the video hashing or comparison operations fail.

pub fn compare_videos2(video_path1: &str, video_path2: &str) -> Result<f64, Box<dyn Error>> {
	let tolerance = 0.8;
	let _unique_frames1 = extract_and_filter_frames1(video_path1, tolerance)?;
	let _unique_frames2 = extract_and_filter_frames1(video_path2, tolerance)?;

	let hash_results = vec![
		VideoHash::from_path(video_path1),
		VideoHash::from_path(video_path2),
	];

	let video_hashes: Vec<VideoHash> = hash_results.into_iter().collect::<Result<_, _>>()?;

	let normalized_tolerance = NormalizedTolerance::default();
	let dup_groups = vid_dup_finder_lib::search(video_hashes, normalized_tolerance);

	let similarity = if dup_groups.len() == 1 { 1.0 } else { 0.0 };

	Ok(similarity)
}
// Your existing frame extraction function
fn extract_frames(video_path: &str) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
	// Implement frame extraction logic here
	Ok(Vec::new()) // Placeholder
}

// Your existing hash frame function
fn hash_frame(frame: &[u8]) -> Vec<u8> {
	let mut hasher = blake3::Hasher::new();
	hasher.update(frame);
	hasher.finalize().as_bytes().to_vec()
}
