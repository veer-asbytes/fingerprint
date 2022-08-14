use std::fs::File;
use std::{
	mem::size_of,
	os::unix::fs::{FileExt, MetadataExt},
	path::PathBuf,
};

use divrem::DivRem;
use rand::{rngs::SmallRng, SeedableRng};

use crate::NUM_FINGERPRINT_SEGMENTS;

use super::{ChooseMultipleStable, Error, FingerElement, FingerSegment, Fingerprinter, RNG_SEED};

/// Fingerprinter for raw files.
#[derive(Debug)]
pub struct RawFingerprinter {
	path: PathBuf,
	handle: File,
	segment_sizes: Vec<usize>,
}

impl<'fp> Fingerprinter<'fp> for RawFingerprinter {
	type Segment = RawSegmentIterator<'fp>;

	fn new<P: AsRef<std::path::Path>>(path: P) -> Result<RawFingerprinter, Error> {
		let path = path.as_ref().to_path_buf();
		let size = path.metadata()?.size() as usize;
		let (segment_size, remainder) = size.div_rem(NUM_FINGERPRINT_SEGMENTS);
		let mut rng: SmallRng = SeedableRng::seed_from_u64(RNG_SEED);
		let mut segment_sizes = vec![segment_size; NUM_FINGERPRINT_SEGMENTS];

		segment_sizes.choose_multiple_stable(&mut rng, segment_size, remainder);

		Ok(Self {
			handle: File::open(&path)?,
			path: path,
			segment_sizes: segment_sizes,
		})
	}

	fn path(&self) -> PathBuf {
		self.path.clone()
	}

	fn segments(&'fp self) -> Self::Segment {
		Self::Segment {
			fp: self,
			index: 0,
			pos: 0,
		}
	}
}

/// Structure for a raw fingerprint segment
#[derive(Debug)]
pub struct RawSegment<'fp> {
	fp: &'fp RawFingerprinter,
	index: usize,
	pos: usize,
	size: usize,
	value: Option<u8>,
}

impl<'fp> FingerSegment<'fp> for RawSegment<'fp> {
	type Fingerprinter = &'fp RawFingerprinter;
	type Element = RawElementIterator<'fp>;
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

	fn value(&mut self) -> Self::Value {
		match self.value {
			Some(value) => value,
			None => {
				let total = self
					.elements()
					.fold(0u128, |total, element| total + element.data() as u128);

				let value = (total / self.size as u128) as u8;

				self.value = Some(value);

				value
			}
		}
	}

	fn elements(&'fp self) -> Self::Element {
		Self::Element {
			fp: self.fp,
			segment: self,
			index: 0,
		}
	}
}

/// Iterator for segments in a raw fingerprint.
#[derive(Clone)]
pub struct RawSegmentIterator<'fp> {
	fp: &'fp RawFingerprinter,
	index: usize,
	pos: usize,
}

impl<'fp> Iterator for RawSegmentIterator<'fp> {
	type Item = RawSegment<'fp>;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.index;
		let start_pos = self.pos;
		let end_pos = start_pos + self.fp.segment_sizes.get(index)?;

		self.index += 1;
		self.pos = end_pos;

		match self.index {
			0..=NUM_FINGERPRINT_SEGMENTS => Some(RawSegment {
				fp: self.fp,
				index: index,
				pos: start_pos,
				size: end_pos - start_pos,
				value: None,
			}),
			_ => None,
		}
	}
}

/// Structure for a single byte (u8) of raw data.
#[derive(Debug)]
pub struct RawElement<'fp> {
	fp: &'fp RawFingerprinter,
	segment: &'fp RawSegment<'fp>,
	index: usize,
	pos: usize,
	size: usize,
	data: u8,
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

	fn data(&self) -> Self::Data {
		self.data
	}
}

/// Iterator for elements in a raw fingerprint segment.
#[derive(Clone)]
pub struct RawElementIterator<'fp> {
	fp: &'fp RawFingerprinter,
	segment: &'fp RawSegment<'fp>,
	index: usize,
}

impl<'fp> Iterator for RawElementIterator<'fp> {
	type Item = RawElement<'fp>;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.index;
		let pos = self.segment.pos + index;
		let mut data = [0u8; 1];

		self.fp.handle.read_exact_at(&mut data, pos as u64).ok()?;
		self.index += 1;

		Some(RawElement {
			fp: self.fp,
			segment: self.segment,
			index: index,
			pos: pos,
			size: size_of::<u8>(),
			data: data[0],
		})
	}
}
