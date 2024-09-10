//! Rust file fingerprinting library, supporting many types of audio/video/image/text file formats.

#![deny(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]
use bitvec::prelude::*;

use std::{
	error,
	fmt::Display,
	path::{Path, PathBuf},
};

use fingerprinters::{raw::RawFingerprinter, Fingerprinter};

/// Dedicated fingerprinters for various file types.
pub mod fingerprinters;
mod vid_finder;
mod vid_finder1;
mod video_fingerprint; // Ensure this module is publicly declared
pub use crate::vid_finder::{compare_videos1, extract_and_filter_frames};
pub use crate::vid_finder1::{compare_videos2, extract_and_filter_frames1};
pub use crate::video_fingerprint::{
	compare_videos5, compare_videos_with_nvdec, generate_fingerprints,
};
/// Number of bits (segments) in fingerprint.
const NUM_FINGERPRINT_SEGMENTS: usize = 128;

/// File types with dedicated fingerprinters.
#[derive(Debug, Clone)]
pub enum Type {
	/// Raw fingerprinter.
	Raw,

	/// Text fingerprinter.
	Text,

	/// Image fingerprinter.
	Image,

	/// Audio fingerprinter.
	Audio,

	/// Video fingerprinter.
	Video,
}

/// Generic [error::Error] type.
type Error = Box<dyn error::Error>;

/// High-level methods for producing deterministic fingerprints for files.
#[derive(Debug, Clone)]
pub struct Fingerprint {
	path: PathBuf,
	fingerprint: BitBox<u8>,
	r#type: Type,
}

impl Fingerprint {
	/// Generate a deterministic fingerprint for a file at the given path.
	pub fn finger<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
		let (fingerprint, kind) = match infer::get_from_path(&path)? {
			Some(kind) => match kind.matcher_type() {
				infer::MatcherType::Text => {
					todo!()
				}
				infer::MatcherType::Image => {
					todo!()
				}
				infer::MatcherType::Audio => {
					todo!()
				}
				infer::MatcherType::Video => {
					// Use the `generate_fingerprints` function here
					let frames =
						video_fingerprint::extract_frames(&path.as_ref().to_string_lossy())?;
					let fingerprints = video_fingerprint::generate_fingerprints(frames);
					(
						BitBox::from_bitslice(&BitSlice::from_slice(&fingerprints.concat())),
						Type::Video,
					)
				}
				_ => (RawFingerprinter::new(&path)?.finger()?, Type::Raw),
			},
			None => (RawFingerprinter::new(&path)?.finger()?, Type::Raw),
		};

		Ok(Self {
			path: path.as_ref().into(),
			fingerprint,
			r#type: kind,
		})
	}
	/// Compares the fingerprint of this instance with another fingerprint.
	///
	/// This method computes a similarity score between two fingerprints. It compares
	/// the bit slices of the two fingerprints and returns a similarity score as a
	/// floating-point number between 0.0 and 1.0. The score represents the fraction
	/// of bits that match. Additionally, the method considers the possibility that
	/// the order of the bits might be reversed, and it returns the maximum similarity
	/// score obtained either from direct comparison or reversed comparison.
	///
	/// # Arguments
	///
	/// * `other` - The `Fingerprint` instance to compare against.
	///
	/// # Returns
	///
	/// Returns a `f64` value representing the similarity score between this fingerprint
	/// and the `other` fingerprint. The score ranges from 0.0 (no similarity) to 1.0
	/// (perfect similarity).
	///
	/// # Example
	///
	/// ```
	/// let first = Fingerprint::finger("path/to/video1.mp4").unwrap();
	/// let second = Fingerprint::finger("path/to/video2.mp4").unwrap();
	/// let similarity = first.compare(&second);
	/// println!("Similarity score: {:.2}", similarity);
	/// ```
	///
	/// # Panics
	///
	/// This method may panic if the bit slices are of different lengths, though this
	/// is handled by taking the minimum length of the two bit slices.
	///
	/// # Notes
	///
	/// If both fingerprints have the same bits but in reverse order, the similarity
	/// score will be adjusted to account for that.
	/// Compare this fingerprint with another. Fingerprints may have different [Fingerprint::type]s.
	pub fn compare(&self, other: &Fingerprint) -> f64 {
		let bits_self = self.bits();
		let bits_other = other.bits();

		let bits_self_slice = bits_self.as_bitslice();
		let bits_other_slice = bits_other.as_bitslice();

		let min_len = bits_self_slice.len().min(bits_other_slice.len());
		let mut matching_bits = 0;

		println!("Comparing fingerprints:");
		println!(
			"Self bits: {:?}",
			bits_self_slice.iter().take(50).collect::<Vec<_>>()
		); // Print first 50 bits
		println!(
			"Other bits: {:?}",
			bits_other_slice.iter().take(50).collect::<Vec<_>>()
		); // Print first 50 bits

		for i in 0..min_len {
			if bits_self_slice[i] == bits_other_slice[i] {
				matching_bits += 1;
			} else {
				println!(
					"Mismatch at position {}: Self: {}, Other: {}",
					i, bits_self_slice[i], bits_other_slice[i]
				);
			}
		}

		let direct_similarity = matching_bits as f64 / min_len as f64;
		println!("Direct similarity: {:.2}", direct_similarity);

		// Return the maximum similarity
		// Return the maximum similarity
		// similarity
		direct_similarity
	}
	//hamminf distance

	/// Return vector of fingerprint bits.
	pub fn bits(&self) -> BitBox<u8> {
		self.fingerprint.clone()
	}

	/// Return vector of fingerprint bytes.
	pub fn bytes(&self) -> &[u8] {
		self.fingerprint.as_raw_slice()
	}

	/// Return path to fingerprinted file.
	pub fn path(&self) -> PathBuf {
		self.path.to_path_buf()
	}

	/// Return type of fingerprinter used.
	pub fn r#type(&self) -> Type {
		self.r#type.clone()
	}
}

impl Display for Fingerprint {
	/// Formats the fingerprint in hexadecimal notation.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", hex::encode(self.bytes()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Fingerprint;
	// use std::fs;

	#[test]
	fn test_fingerprint_comparison() {
		// Ensure you have sample videos in the specified paths for testing
		let video1_path = "samples/vid.mp4";
		let video2_path = "samples/lesson4.mp4";

		// Extract frames and handle potential errors
		// let frames1 = match video_fingerprint::extract_frames(video1_path) {
		// 	Ok(frames) => frames,
		// 	Err(e) => {
		// 		eprintln!("Error extracting frames from {}: {}", video1_path, e);
		// 		return;
		// 	}
		// };
		//
		// let frames2 = match video_fingerprint::extract_frames(video2_path) {
		// 	Ok(frames) => frames,
		// 	Err(e) => {
		// 		eprintln!("Error extracting frames from {}: {}", video2_path, e);
		// 		return;
		// 	}
		// };
		//
		// // Generate fingerprints and handle potential errors
		// let fingerprints1 = video_fingerprint::generate_fingerprints(frames1);
		// let fingerprints2 = video_fingerprint::generate_fingerprints(frames2);

		// Perform comparison and handle potential errors
		match video_fingerprint::compare_videos_with_nvdec(video1_path, video2_path) {
			Ok(similarity) => {
				println!("Similarity score: {}", similarity);

				assert!(similarity > 0.0, "Videos are not similar");
			}
			Err(e) => {
				eprintln!("Error comparing videos: {}", e);
				panic!("Test failed due to error");
			}
		}
	}

	#[test]
	fn test_empty() {
		assert_eq!(
			Fingerprint::finger("samples/empty").unwrap().to_string(),
			"51ad9acc76659b1a4d4da56055b1b532"
		);
	}

	#[test]
	fn test_ascii_text() {
		assert_eq!(
			Fingerprint::finger("samples/ascii.txt")
				.unwrap()
				.to_string(),
			"6964d14b3a2bf3264db15649d5de4ad5"
		);
	}

	#[test]
	fn test_ascii_text_similar() {
		let first = Fingerprint::finger("samples/ascii.txt").unwrap();
		let second = Fingerprint::finger("samples/ascii_similar.txt").unwrap();

		assert_eq!(first.compare(&second), 0.859375);
	}

	#[test]
	fn test_ascii_text_somewhat_similar() {
		let first = Fingerprint::finger("samples/3.mp4").unwrap();
		let second = Fingerprint::finger("samples/4.mp4").unwrap();
		// Compare the fingerprints
		let similarity = first.compare(&second);

		// Print the similarity score
		println!(
			"Similarity score between 'samples/lesson4.mp4' and 'samples/vid.mp4': {:.2}",
			similarity
		);

		// Assertion
		assert_eq!(similarity, 1.0);
	}

	#[test]
	fn test_ascii_text_different() {
		let first = Fingerprint::finger("samples/ascii.txt").unwrap();
		let second = Fingerprint::finger("samples/ascii_different.txt").unwrap();

		assert_eq!(first.compare(&second), 0.4921875);
	}
}
