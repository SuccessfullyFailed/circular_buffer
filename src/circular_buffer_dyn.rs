/// Works the same as CircularBuffer, but uses a vec instead of an array.
/// This allows it to be sized dynamically in the 'new' function.
/// Despite that, the buffer will not move as its size is constant throughout its entire lifetime.
/// This can not be used statically, but does perform better than a normal Vec, as the list does not change in size, allowing it to stay in the same place in memory.
#[derive(PartialEq, Eq, Clone)]
pub struct CircularBufferDyn<T> {
	buffer:Vec<T>,
	capacity:usize, // Same as buffer.len(), but dynamically fetching is not useful as the buffer length always stays the same.
	read_cursor:usize,
	write_cursor:usize
}
impl<T:Default + Clone> CircularBufferDyn<T> {
	
	/* CONSTRUCTOR METHODS */

	/// Create a new circular-buffer.
	pub fn new(capacity:usize) -> CircularBufferDyn<T> {
		CircularBufferDyn {
			buffer: vec![T::default(); capacity],
			capacity,
			read_cursor: 0,
			write_cursor: 0
		}
	}



	/* BUFFER METHODS */

	/// Add a single sample to the buffer. Returns the amount of samples stored to the buffer.
	pub fn push(&mut self, input:T) -> usize {
		self.extend(&[input])
	}

	/// Add a list of samples to the buffer. Returns the amount of samples stored to the buffer.
	pub fn extend(&mut self, input:&[T]) -> usize {

		// Find out how much free space is left before wrap.
		let used_space:usize = self.len();
		let available_space:usize = self.capacity - used_space;
		let required_space:usize = input.len();

		// If input is too large, only write beginning. Always keep one "empty" slot. This makes sure both cursors with the same value always means the buffer is empty, rather than full.
		if required_space >= available_space {
			return self.extend(&input[..available_space - 1]);
		}

		// If not enough space before wrap, return or split into two modifications.
		let available_space_before_wrap:usize = self.capacity - self.write_cursor;
		if available_space_before_wrap < required_space {
			return self.extend(&input[..available_space_before_wrap]) + self.extend(&input[available_space_before_wrap..required_space]);
		}

		// If enough space before wrap, write to buffer.
		self.buffer[self.write_cursor..self.write_cursor + required_space].clone_from_slice(&input);
		self.write_cursor = (self.write_cursor + required_space) % self.capacity;
		required_space
	}

	/// Take one sample from the buffer.
	pub fn take_one(&mut self) -> T {
		let found:Vec<T> = self.take(1);
		if found.is_empty() {
			T::default()
		} else {
			found[0].clone()
		}
	}

	/// Take all remaining samples from the buffer.
	pub fn take_all(&mut self) -> Vec<T> {
		self.take(self.len())
	}

	/// Take an amount of samples from the buffer.
	pub fn take(&mut self, amount:usize) -> Vec<T> {
		let mut output_buffer:Vec<T> = vec![T::default(); amount];
		let written_amount:usize = self.take_to_buffer(&mut output_buffer);
		output_buffer[..written_amount].to_vec()
	}

	/// Take an amount of samples from the buffer. Writes the data to the given output. Returns the amount of data taken from the buffer.
	fn take_to_buffer(&mut self, output:&mut [T]) -> usize {

		// Find out how much free space is left before wrap.
		let used_space:usize = self.len();
		if used_space == 0 {
			return 0;
		}
		let used_required_space:usize = used_space.min(output.len());

		// Take straight part.
		let straight_space:usize = used_required_space.min(self.capacity - self.read_cursor);
		output[..straight_space].clone_from_slice(&self.buffer[self.read_cursor..self.read_cursor + straight_space]);
		self.read_cursor += straight_space;
		let wrapped_space:usize = used_required_space - straight_space;
		if wrapped_space != 0 {
			self.read_cursor -= self.capacity;
			output[straight_space..straight_space + wrapped_space].clone_from_slice(&self.buffer[self.read_cursor..self.read_cursor + wrapped_space]);
			self.read_cursor += wrapped_space;
		}
		
		// Return taken amount.
		straight_space + wrapped_space
	}

	/// Get all data that is written in the buffer, including the amount already having been read. The newest samples will be at the end of the list.
	pub fn raw_data(&self) -> Vec<T> {
		let mut output:Vec<T> = self.buffer.to_vec();
		output.rotate_left(self.read_cursor);
		output
	}



	
	/* PROPERTY GETTER METHODS */

	/// Return the amount of currently stored samples.
	pub fn len(&self) -> usize {
		if self.write_cursor >= self.read_cursor {
			self.write_cursor - self.read_cursor
		} else {
			self.capacity - (self.read_cursor - self.write_cursor)
		}.min(self.capacity)
	}

	/// Wether or not there are 0 stored samples.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Wether or not the buffer is full.
	pub fn is_full(&self) -> bool {
		self.len() == self.capacity - 1
	}
}