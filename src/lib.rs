//! Rust file fingerprinting library, supporting many types of audio/video/image/text file formats.

#![deny(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

use std::{
	error,
	fmt::Display,
	path::{Path, PathBuf},
};

use bitvec::prelude::*;

use fingerprinters::{raw::RawFingerprinter, Fingerprinter};

/// Dedicated fingerprinters for various file types.
pub mod fingerprinters;

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
					todo!()
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

	/// Compare this fingerprint with another. Fingerprints may have different [Fingerprint::type]s.
	pub fn compare(&self, other: &Fingerprint) -> f64 {
		let mut similarity = 0f64;

		for (lbit, rbit) in self.bits().iter().zip(other.bits().iter()) {
			if lbit == rbit {
				similarity += 1f64;
			}
		}

		similarity / NUM_FINGERPRINT_SEGMENTS as f64
	}

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
	use crate::Fingerprint;

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
		let first = Fingerprint::finger("samples/ascii.txt").unwrap();
		let second = Fingerprint::finger("samples/ascii_somewhat_similar.txt").unwrap();

		assert_eq!(first.compare(&second), 0.65625);
	}

	#[test]
	fn test_ascii_text_different() {
		let first = Fingerprint::finger("samples/ascii.txt").unwrap();
		let second = Fingerprint::finger("samples/ascii_different.txt").unwrap();

		assert_eq!(first.compare(&second), 0.4921875);
	}
}
