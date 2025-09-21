pub struct CircularBuffer<T, const CAPACITY:usize> {
	buffer:[T; CAPACITY],
	read_cursor:usize,
	write_cursor:usize
}
impl<T:Copy, const CAPACITY:usize> CircularBuffer<T, CAPACITY> {

	/// Create a new circular buffer as compile-time constant.
	pub const fn new_const(default_value:T) -> CircularBuffer<T, CAPACITY> {
		CircularBuffer {
			buffer: [default_value; CAPACITY],
			read_cursor: 0,
			write_cursor: 0
		}
	}
}
impl<T:Default + Copy, const CAPACITY:usize> CircularBuffer<T, CAPACITY> {
	
	/* CONSTRUCTOR METHODS */

	/// Create a new circular-buffer.
	pub fn new() -> CircularBuffer<T, CAPACITY> {
		CircularBuffer {
			buffer: [T::default(); CAPACITY],
			read_cursor: 0,
			write_cursor: 0
		}
	}



	/* BUFFER METHODS */

	/// Add a list of samples to the buffer. Returns the amount of samples stored to the buffer.
	pub fn extend(&mut self, input:&[T]) -> usize {

		// Find out how much free space is left before wrap.
		let used_space:usize = self.len();
		let available_space:usize = CAPACITY - used_space;
		let required_space:usize = input.len();

		// If input is too large, only write beginning. Always keep one "empty" slot. This makes sure both cursors with the same value always means the buffer is empty, rather than full.
		if required_space >= available_space {
			return self.extend(&input[..available_space - 1]);
		}

		// If not enough space before wrap, return or split into two modifications.
		let available_space_before_wrap:usize = CAPACITY - self.write_cursor;
		if available_space_before_wrap < required_space {
			return self.extend(&input[..available_space_before_wrap]) + self.extend(&input[available_space_before_wrap..required_space]);
		}

		// If enough space before wrap, write to buffer.
		self.buffer[self.write_cursor..self.write_cursor + required_space].copy_from_slice(&input);
		self.write_cursor = (self.write_cursor + required_space) % CAPACITY;
		required_space
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
		let straight_space:usize = used_required_space.min(CAPACITY - self.read_cursor);
		output[..straight_space].copy_from_slice(&self.buffer[self.read_cursor..self.read_cursor + straight_space]);
		self.read_cursor += straight_space;
		let wrapped_space:usize = used_required_space - straight_space;
		if wrapped_space != 0 {
			self.read_cursor -= CAPACITY;
			output[straight_space..straight_space + wrapped_space].copy_from_slice(&self.buffer[self.read_cursor..self.read_cursor + wrapped_space]);
			self.read_cursor += wrapped_space;
		}
		
		// Return taken amount.
		straight_space + wrapped_space
	}

	/// Get all data that is written in the buffer, ignoring the amount already having been read.
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
			CAPACITY - (self.read_cursor - self.write_cursor)
		}.min(CAPACITY)
	}

	/// Wether or not there are 0 stored samples.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Wether or not the buffer is full.
	pub fn is_full(&self) -> bool {
		self.len() == CAPACITY - 1
	}
}