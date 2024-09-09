use std::collections::HashSet;
use std::error::Error;
use vid_dup_finder_lib::{NormalizedTolerance, VideoHash};

/// Extracts frames from a video and filters out similar frames based on hashing.
///
/// # Arguments
///
/// * `video_path` - The path to the video file.
/// * `tolerance` - The tolerance level for determining similarity between frames.
///
/// # Returns
///
/// This function returns a `Result` containing a vector of unique frames if successful,
/// or an error if something goes wrong.
///
/// # Errors
///
/// This function will return an error if the frame extraction fails or if any other error occurs.
pub fn extract_and_filter_frames(
	video_path: &str,
	tolerance: f64,
) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
	// Placeholder for your frame extraction code
	let frames = extract_frames(video_path)?;

	// Filter out similar frames based on hashing
	let mut unique_frames = Vec::new();
	let mut seen_hashes = HashSet::new();

	for frame in frames {
		let hash = hash_frame(&frame);
		let hash_str = hex::encode(&hash);

		if !seen_hashes.contains(&hash_str) {
			seen_hashes.insert(hash_str.clone());
			unique_frames.push(frame);
		}
	}

	Ok(unique_frames)
}

/// Compares two videos for similarity.
///
/// # Arguments
///
/// * `video_path1` - The path to the first video file.
/// * `video_path2` - The path to the second video file.
///
/// # Returns
///
/// This function returns a `Result` containing a similarity score between 0.0 and 1.0 if successful,
/// or an error if something goes wrong.
///
/// # Errors
///
/// This function will return an error if video hashing fails or if any other error occurs.
pub fn compare_videos1(video_path1: &str, video_path2: &str) -> Result<f64, Box<dyn Error>> {
	// Generate hashes for the entire videos
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
