//! Rust file fingerprinting library, supporting many types of audio/video/image/text file formats.

#![deny(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

use std::{
	fmt::Display,
	path::{Path, PathBuf},
};

/// Fingerprint represents a 64-byte fingerprint.
#[derive(Debug)]
pub struct Fingerprint {
	data: [u8; 64],
}

impl Display for Fingerprint {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for byte in self.data {
			write!(f, "{:x?}", byte)?;
		}

		Ok(())
	}
}

/// Fingerprinter provides methods for generating fingerprints.
#[derive(Debug)]
pub struct Fingerprinter {
	path: PathBuf,
}

impl Fingerprinter {
	/// Generate a fingerprint for the given path.
	pub fn finger<P: AsRef<Path>>(path: P) -> Fingerprint {
		/*Self {
			path: path.as_ref().into(),
		}*/

		return Fingerprint {
			data: [0xff].repeat(64).try_into().unwrap(),
		};
	}
}

#[cfg(test)]
mod tests {
	use crate::Fingerprinter;

	#[test]
	fn test_fingerprinter() {
		let fp = Fingerprinter::finger("test.mkv");

		println!("{}", fp);
	}
}
