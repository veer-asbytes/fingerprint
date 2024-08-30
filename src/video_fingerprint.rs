use ffmpeg_next::{codec, format, frame, media, packet, Error};
use sha2::{Digest, Sha256};

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

	// one way
	let codec = codec::Id::from(codec_params.id());
	let mut decoder = codec::Context::from_parameters(codec_params)?
		.decoder()
		.video()?;
	// let codec = codec_params.decoder().ok_or(Error::DecoderNotFound)?;

	//other way
	// let mut context = codec::Context::from_parameters(codec_params)?;
	// let mut decoder = context.open_as(codec)?;
	// Initialize the codec context

	//other way
	// let context_decoder = codec::context::Context::from_parameters(codec_params)?;
	// let mut decoder = context_decoder.decoder().video()?;
	// Initialize a frame container
	let mut frame = frame::Video::empty();

	let mut frames = Vec::new();

	// Process packets and extract frames
	for (stream, packet) in ictx.packets() {
		if stream.index() == input_stream_index {
			decoder.send_packet(&packet)?;
			while let Ok(()) = decoder.receive_frame(&mut frame) {
				let frame_data = frame.data(0).to_vec();
				frames.push(frame_data);
			}
		}
	}

	Ok(frames)
}

pub fn hash_frame(frame: &[u8]) -> Vec<u8> {
	let mut hasher = Sha256::new();
	hasher.update(frame);
	hasher.finalize().to_vec()
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
	frames.into_iter().map(|frame| hash_frame(&frame)).collect()
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
	let frames1 = extract_frames(video_path1)?;
	let frames2 = extract_frames(video_path2)?;

	let fingerprints1 = generate_fingerprints(frames1);
	let fingerprints2 = generate_fingerprints(frames2);

	// Simple similarity check: count matching fingerprints
	let mut matches = 0;
	let total = fingerprints1.len().min(fingerprints2.len());

	for fingerprint1 in &fingerprints1 {
		if fingerprints2.contains(fingerprint1) {
			matches += 1;
		}
	}

	Ok(matches as f64 / total as f64)
}
