//! Rust file fingerprinting library, supporting many types of audio/video/image/text file formats.

#![deny(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

use std::{
	error,
	fmt::Display,
	path::{Path, PathBuf},
};

use bitvec::prelude::*;
use hex;
use infer;

use fingerprinters::{raw::RawFingerprinter, FingerElement, FingerSegment, Fingerprinter};

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
	fp_bits: BitBox<u8>,
	r#type: Type,
}

impl Fingerprint {
	/// Generate a deterministic fingerprint for a file at the given path.
	pub fn finger<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
		let (fp_bits, kind) = match infer::get_from_path(&path)? {
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
				_ => (Self::process(&RawFingerprinter::new(&path)?), Type::Raw),
			},
			None => (Self::process(&RawFingerprinter::new(&path)?), Type::Raw),
		};

		Ok(Self {
			path: path.as_ref().into(),
			fp_bits: fp_bits,
			r#type: kind,
		})
	}

	/// Process through each segment of a file using a particular fingerprinter, generating the final fingerprint.
	fn process<'fp, F>(fp: &'fp F) -> BitBox<u8>
	where
		F: Fingerprinter<'fp>,
		<F::SegmentIter as Iterator>::Item: FingerSegment<'fp>,
	{
		let mut fp_bits = bitbox![u8, Lsb0; 0; 128];
		let mut last = None;
		let mut bit_index = 0;

		for mut segment in fp.segments().cycle() {
			let value = segment.value();

			if let Some(last) = last {
				if value > last {
					fp_bits.set(bit_index, true);
				}

				bit_index += 1;

				if bit_index >= 128 {
					break;
				}
			}

			last = Some(value);
		}

		fp_bits
	}

	/// Return vector of fingerprint bits.
	pub fn bits(&self) -> BitBox<u8> {
		self.fp_bits.clone()
	}

	/// Return vector of fingerprint bytes.
	pub fn bytes(&self) -> &[u8] {
		self.fp_bits.as_raw_slice()
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
	fn test_raw() {
		let fp = Fingerprint::finger("LICENSE").unwrap();

		assert_eq!(fp.to_string(), "6a6ed537622bd136559056d58a55c9d2");
	}
}
