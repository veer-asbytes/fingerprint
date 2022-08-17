use std::{
	io,
	path::{Path, PathBuf},
};

use bitvec::prelude::*;
use rand::prelude::*;

use crate::{Error, NUM_FINGERPRINT_SEGMENTS};

/// Implementation of raw fingerprinter.
pub mod raw;

/// Seed for deterministic RNG.
const RNG_SEED: u64 = 939270607250626829;

/// Provides RNG support methods.
trait ChooseMultipleStable {
	/// Produce stable (deterministic) RNG for fingerprint segment sizing.
	fn choose_multiple_stable<R>(
		&mut self,
		rng: &mut R,
		initial_segment_size: usize,
		remainder: usize,
	) -> &Vec<usize>
	where
		R: Rng + ?Sized;
}

impl ChooseMultipleStable for Vec<usize> {
	fn choose_multiple_stable<R>(
		&mut self,
		rng: &mut R,
		initial_segment_size: usize,
		mut remainder: usize,
	) -> &Vec<usize>
	where
		R: Rng + ?Sized,
	{
		let mut index;

		while remainder > 0 {
			index = rng.gen_range(0..self.len());

			if let Some(value) = self.get_mut(index) {
				if *value == initial_segment_size {
					*value += 1;
					remainder -= 1;
				}
			}
		}

		self
	}
}

/// Contract of methods implementing a fingerprinter.
pub trait Fingerprinter<'fp>
where
	&'fp Self: 'fp + IntoIterator,
	<&'fp Self as IntoIterator>::Item: FingerSegment<'fp>,
	&'fp <&'fp Self as IntoIterator>::Item: IntoIterator,
	<&'fp <&'fp Self as IntoIterator>::Item as IntoIterator>::Item: FingerElement,
{
	/// Create new fingerprinter.
	fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error>
	where
		Self: Sized;

	/// Return path of file being fingerprinted.
	fn path(&self) -> PathBuf;

	/// Process through each segment of a file using a particular fingerprinter, generating the final fingerprint.
	fn finger(&'fp self) -> Result<BitBox<u8>, Error> {
		let mut fingerprint = bitbox![u8, Lsb0; 0; NUM_FINGERPRINT_SEGMENTS];
		let mut first = None;
		let mut last = None;

		for mut segment in self {
			let value = segment.value()?;

			match last {
				Some(last) => {
					if value >= last {
						fingerprint.set(segment.index() - 1, true);
					}
				}
				None => {
					first = Some(segment.value()?);
				}
			}

			last = Some(value);
		}

		if first.ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?
			>= last.ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?
		{
			fingerprint.set(NUM_FINGERPRINT_SEGMENTS - 1, true);
		}

		Ok(fingerprint)
	}
}

/// Methods for a fingerprint segment. A fingerprint consists of a fixed number of segments.
pub trait FingerSegment<'fp>
where
	&'fp Self: 'fp + IntoIterator,
	<&'fp Self as IntoIterator>::Item: FingerElement,
{
	/// Type of fingerprinter.
	type Fingerprinter;

	/// Type of fingerprint segment value.
	type Value: PartialOrd;

	/// Returns fingerprinter.
	fn fingerprinter(&self) -> Self::Fingerprinter;

	/// Returns the index of the current segment.
	fn index(&self) -> usize;

	/// Returns the file position for the current segment.
	fn pos(&self) -> usize;

	/// Returns the size (bytes) of the current segment.
	fn size(&self) -> usize;

	/// Returns the segment value.
	fn value(&mut self) -> Result<Self::Value, Error>;
}

/// Methods for an element contained in a fingerprint segment.
pub trait FingerElement {
	/// Type of fingerprinter.
	type Fingerprinter;

	/// Type of fingerprint segment.
	type Segment;

	/// Type of element data.
	type Data;

	/// Returns fingerprinter.
	fn fingerprinter(&self) -> Self::Fingerprinter;

	/// Returns fingerprint segment.
	fn segment(&self) -> Self::Segment;

	/// Returns index of the element relative to the segment.
	fn index(&self) -> usize;

	/// Returns the file position for the current element.
	fn pos(&self) -> usize;

	/// Returns the size (bytes) of the current element.
	fn size(&self) -> usize;

	/// Returns the value of the element.
	fn data(&self) -> Result<Self::Data, Error>;
}
