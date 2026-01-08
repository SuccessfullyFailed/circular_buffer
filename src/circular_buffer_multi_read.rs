use crate::ReadCursor;



/// Works the same as CircularBuffer, but allows using multiple threads to read.
/// For each thread that wants to read from the buffer, create a cursor that keeps track of that cursors' last read values.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CircularBufferMultiRead<T, const CAPACITY:usize, const MAX_READ_CURSOR_COUNT:usize> {
	buffer:[T; CAPACITY],
	read_cursors:[usize; MAX_READ_CURSOR_COUNT],
	current_read_cursor_count:usize,
	write_cursor:usize
}
impl<T:Copy, const CAPACITY:usize, const MAX_READ_CURSOR_COUNT:usize> CircularBufferMultiRead<T, CAPACITY, MAX_READ_CURSOR_COUNT> {

	/// Create a new circular buffer as compile-time constant.
	pub const fn new_const(default_value:T) -> CircularBufferMultiRead<T, CAPACITY, MAX_READ_CURSOR_COUNT> {
		CircularBufferMultiRead {
			buffer: [default_value; CAPACITY],
			read_cursors: [0; MAX_READ_CURSOR_COUNT],
			current_read_cursor_count: 0,
			write_cursor: 0
		}
	}
}
impl<T:Default + Copy, const CAPACITY:usize, const MAX_READ_CURSOR_COUNT:usize> CircularBufferMultiRead<T, CAPACITY, MAX_READ_CURSOR_COUNT> {
	
	/* CONSTRUCTOR METHODS */

	/// Create a new circular-buffer.
	pub fn new() -> CircularBufferMultiRead<T, CAPACITY, MAX_READ_CURSOR_COUNT> {
		CircularBufferMultiRead {
			buffer: [T::default(); CAPACITY],
			read_cursors: [0; MAX_READ_CURSOR_COUNT],
			current_read_cursor_count: 0,
			write_cursor: 0
		}
	}



	/* BUFFER WRITING METHODS */

	/// Create a ReadCursor.
	pub fn create_read_cursor<'a>(&'a mut self) -> ReadCursor {
		let cursor_id:usize = self.current_read_cursor_count;
		if cursor_id > MAX_READ_CURSOR_COUNT {
			panic!("Could not create CircularBufferMultiRead Cursor, max cursor count overflow.");
		}
		self.current_read_cursor_count += 1;
		self.read_cursors[cursor_id] = self.write_cursor;
		ReadCursor(cursor_id)
	}

	/// Skip a cursor to the end of data, ignoring all current data.
	pub fn skip_current_data(&mut self, cursor:&ReadCursor) {
		self.read_cursors[cursor.0] = self.write_cursor;
	}

	/// Add a single sample to the buffer. Returns the amount of samples stored to the buffer.
	pub fn push(&mut self, input:T) -> usize {
		self.extend(&[input])
	}

	/// Add a list of samples to the buffer. Returns the amount of samples stored to the buffer.
	pub fn extend(&mut self, input:&[T]) -> usize {

		// Find out how much free space is left before wrap.
		let largest_used_space:usize = (0..self.current_read_cursor_count).map(|cursor_index| self.len(&ReadCursor(cursor_index))).max().unwrap_or_default();
		let available_space:usize = CAPACITY - largest_used_space;
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



	/* BUFFER READING METHODS */

	/// Get all data that is written in the buffer, including the amount already having been read.
	pub fn raw_data(&self, read_cursor:&ReadCursor) -> Vec<T> {
		let mut output:Vec<T> = self.buffer.to_vec();
		output.rotate_left(self.read_cursors[read_cursor.0]);
		output
	}

	/// Take one sample from the buffer.
	pub fn take_one(&mut self, read_cursor:&ReadCursor) -> T {
		let found:Vec<T> = self.take(1, read_cursor);
		if found.is_empty() {
			T::default()
		} else {
			found[0]
		}
	}

	/// Take all remaining samples from the buffer.
	pub fn take_all(&mut self, read_cursor:&ReadCursor) -> Vec<T> {
		self.take(self.len(read_cursor), read_cursor)
	}

	/// Take an amount of samples from the buffer.
	pub fn take(&mut self, amount:usize, read_cursor:&ReadCursor) -> Vec<T> {
		let mut output_buffer:Vec<T> = vec![T::default(); amount];
		let written_amount:usize = self.take_to_buffer(&mut output_buffer, read_cursor);
		output_buffer[..written_amount].to_vec()
	}

	/// Take an amount of samples from the buffer. Writes the data to the given output. Returns the amount of data taken from the buffer.
	pub fn take_to_buffer(&mut self, output:&mut [T], read_cursor_ref:&ReadCursor) -> usize {
		let mut read_cursor:usize = self.read_cursors[read_cursor_ref.0];

		// Find out how much free space is left before wrap.
		let used_space:usize = self.len(read_cursor_ref);
		if used_space == 0 {
			return 0;
		}
		let used_required_space:usize = used_space.min(output.len());

		// Take straight part.
		let straight_space:usize = used_required_space.min(CAPACITY - read_cursor);
		output[..straight_space].copy_from_slice(&self.buffer[read_cursor..read_cursor + straight_space]);
		read_cursor += straight_space;
		let wrapped_space:usize = used_required_space - straight_space;
		if wrapped_space != 0 {
			read_cursor -= CAPACITY;
			output[straight_space..straight_space + wrapped_space].copy_from_slice(&self.buffer[read_cursor..read_cursor + wrapped_space]);
			read_cursor += wrapped_space;
		}
		
		// Return taken amount.
		self.read_cursors[read_cursor_ref.0] = read_cursor;
		straight_space + wrapped_space
	}



	
	/* PROPERTY GETTER METHODS */

	/// Return the amount of unread samples stored for for a specific cursor.
	pub fn len(&self, cursor:&ReadCursor) -> usize {
		let read_cursor:usize = self.read_cursors[cursor.0];
		if self.write_cursor >= read_cursor {
			self.write_cursor - read_cursor
		} else {
			CAPACITY - (read_cursor - self.write_cursor)
		}.min(CAPACITY)
	}

	/// Wether or not there are 0 stored samples.
	pub fn is_empty(&self, cursor:&ReadCursor) -> bool {
		self.len(cursor) == 0
	}

	/// Wether or not the buffer is full.
	pub fn is_full(&self, cursor:&ReadCursor) -> bool {
		self.len(cursor) == CAPACITY - 1
	}
}