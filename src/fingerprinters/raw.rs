use std::fs::File;
use std::sync::Arc;
use std::{
	error,
	mem::size_of,
	os::unix::fs::{FileExt, MetadataExt},
	path::PathBuf,
};

use divrem::DivRem;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::NUM_FINGERPRINT_SEGMENTS;

use super::{ChooseMultipleStable, Error, FingerElement, FingerSegment, Fingerprinter, RNG_SEED};

/// Fingerprinter for raw files.
#[derive(Debug)]
pub struct RawFingerprinter {
	path: PathBuf,
	handle: File,
	rng: ChaCha8Rng,
	segment_sizes: Vec<usize>,
}

impl<'fp> Fingerprinter<'fp> for RawFingerprinter {
	fn new<P: AsRef<std::path::Path>>(path: P) -> Result<RawFingerprinter, Error> {
		let path = path.as_ref().to_path_buf();
		let size = path.metadata()?.size() as usize;
		let (segment_size, remainder) = size.div_rem(NUM_FINGERPRINT_SEGMENTS);
		let mut rng = ChaCha8Rng::seed_from_u64(RNG_SEED);
		let mut segment_sizes = vec![segment_size; NUM_FINGERPRINT_SEGMENTS];

		segment_sizes.choose_multiple_stable(&mut rng, segment_size, remainder);

		Ok(Self {
			handle: File::open(&path)?,
			rng,
			path,
			segment_sizes,
		})
	}

	fn path(&self) -> PathBuf {
		self.path.clone()
	}
}

impl<'fp> IntoIterator for &'fp RawFingerprinter {
	type Item = RawSegment<'fp>;
	type IntoIter = RawSegmentIterator<'fp>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter {
			fp: self,
			index: 0,
			pos: 0,
			rng: self.rng.clone(),
		}
	}
}

/// Structure for a raw fingerprint segment
#[derive(Clone, Debug)]
pub struct RawSegment<'fp> {
	fp: &'fp RawFingerprinter,
	index: usize,
	pos: usize,
	size: usize,
	value: Option<Result<u8, Arc<dyn error::Error>>>,
}

impl<'fp> FingerSegment<'fp> for RawSegment<'fp> {
	type Fingerprinter = &'fp RawFingerprinter;
	type Value = u8;

	fn fingerprinter(&self) -> Self::Fingerprinter {
		self.fp
	}

	fn index(&self) -> usize {
		self.index
	}

	fn pos(&self) -> usize {
		self.pos
	}

	fn size(&self) -> usize {
		self.size
	}

	fn value(&mut self) -> Result<Self::Value, Error> {
		match &self.value {
			Some(value) => match value.clone() {
				Ok(data) => Ok(data),
				Err(e) => Err(Box::new(e)),
			},
			None => {
				let total = self.into_iter().try_fold(0u128, |total, element| {
					Ok::<u128, Error>(total + element.data()? as u128)
				})?;

				let value = (total / self.size as u128) as u8;

				self.value = Some(Ok(value));

				Ok(value)
			}
		}
	}
}

impl<'fp> IntoIterator for &'fp RawSegment<'fp> {
	type Item = RawElement<'fp>;
	type IntoIter = RawElementIterator<'fp>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter {
			fp: self.fp,
			segment: self,
			index: 0,
		}
	}
}

/// Iterator for segments in a raw fingerprint.
#[derive(Clone, Debug)]
pub struct RawSegmentIterator<'fp> {
	fp: &'fp RawFingerprinter,
	index: usize,
	pos: usize,
	rng: ChaCha8Rng,
}

impl<'fp> Iterator for RawSegmentIterator<'fp> {
	type Item = RawSegment<'fp>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= NUM_FINGERPRINT_SEGMENTS {
			return None;
		}

		let index = self.index;
		let start_pos = self.pos;
		let end_pos = start_pos + self.fp.segment_sizes.get(index)?;
		let size = end_pos - start_pos;

		self.index += 1;
		self.pos = end_pos;

		Some(RawSegment {
			fp: self.fp,
			index,
			pos: start_pos,
			size,
			value: match size {
				0 => Some(Ok(self.rng.gen())),
				_ => None,
			},
		})
	}
}

/// Structure for a single byte (u8) of raw data.
#[derive(Clone, Debug)]
pub struct RawElement<'fp> {
	fp: &'fp RawFingerprinter,
	segment: &'fp RawSegment<'fp>,
	index: usize,
	pos: usize,
	size: usize,
	data: Result<u8, Arc<dyn error::Error>>,
}

impl<'fp> FingerElement for RawElement<'fp> {
	type Fingerprinter = &'fp RawFingerprinter;
	type Segment = &'fp RawSegment<'fp>;
	type Data = u8;

	fn fingerprinter(&self) -> Self::Fingerprinter {
		self.fp
	}

	fn segment(&self) -> Self::Segment {
		self.segment
	}

	fn index(&self) -> usize {
		self.index
	}

	fn pos(&self) -> usize {
		self.pos
	}

	fn size(&self) -> usize {
		self.size
	}

	fn data(&self) -> Result<Self::Data, Error> {
		match self.data.clone() {
			Ok(data) => Ok(data),
			Err(e) => Err(Box::new(e)),
		}
	}
}

/// Iterator for elements in a raw fingerprint segment.
#[derive(Clone, Debug)]
pub struct RawElementIterator<'fp> {
	fp: &'fp RawFingerprinter,
	segment: &'fp RawSegment<'fp>,
	index: usize,
}

impl<'fp> Iterator for RawElementIterator<'fp> {
	type Item = RawElement<'fp>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= self.segment.size {
			return None;
		}

		let index = self.index;
		let pos = self.segment.pos + index;
		let mut data = [0u8; 1];

		let data: Result<u8, Arc<dyn error::Error>> =
			match self.fp.handle.read_exact_at(&mut data, pos as u64) {
				Ok(_) => Ok(data[0]),
				Err(e) => {
					//
					Err(Arc::new(e))
				}
			};

		self.index += 1;

		Some(RawElement {
			fp: self.fp,
			segment: self.segment,
			index,
			pos,
			size: size_of::<u8>(),
			data,
		})
	}
}
