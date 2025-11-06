#[cfg(test)]
mod tests {
	use std::time::{ Duration, Instant };
	use crate::CircularBufferMultiread;
	
	

	const TEST_CAPACITY:usize = 8;
	fn get_test_buffer() -> CircularBufferMultiread<i32, TEST_CAPACITY> {
		CircularBufferMultiread::new()
	}



	#[test]
	fn test_new_bufferfer_is_empty() {
		let buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();
		assert_eq!(buffer.len(), 0);
		assert!(buffer.is_empty());
		assert!(!buffer.is_full());
	}

	#[test]
	fn test_extend_and_take_simple() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();

		// Test write.
		let written:usize = buffer.extend(&[1, 2, 3]);
		assert_eq!(written, 3);
		assert_eq!(buffer.len(), 3);

		// Test take.
		assert_eq!(buffer.take(3), vec![1, 2, 3]);
		assert!(buffer.is_empty());
	}

	#[test]
	fn test_extend_over_capacity_truncates() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();

		let written:usize = buffer.extend(&(0..20).collect::<Vec<i32>>());
		assert_eq!(written, TEST_CAPACITY - 1); // Should always keep one "empty" slot. This makes sure both cursors with the same value always means the bufferfer is empty, rather than full.
		assert!(buffer.is_full());
		assert_eq!(buffer.len(), TEST_CAPACITY - 1);
	}

	#[test]
	fn test_take_more_than_available() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();

		buffer.extend(&[1, 2, 3]);
		let taken_data:Vec<i32> = buffer.take(10);
		assert_eq!(taken_data, vec![1, 2, 3]);
		assert!(buffer.is_empty());
	}

	#[test]
	fn test_wraparound_behavior() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();

		// First take.
		let written:usize = buffer.extend(&[1, 2, 3, 4, 5, 6, 7]);
		assert_eq!(written, 7);
		assert!(buffer.is_full());
		assert_eq!(buffer.take(4), vec![1, 2, 3, 4]);
		assert_eq!(buffer.len(), 3);
		
		// Second take.
		let written:usize = buffer.extend(&[8, 9, 10]);
		assert_eq!(buffer.len(), 6);
		assert_eq!(written, 3);
		assert_eq!(buffer.take(6), vec![5, 6, 7, 8, 9, 10]);
		assert!(buffer.is_empty());
	}

	#[test]
	fn test_multiple_small_writes_and_reads() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();
		for i in 0..5 {
			assert_eq!(buffer.extend(&[i]), 1);
		}
		assert_eq!(buffer.len(), 5);

		for i in 0..5 {
			assert_eq!(buffer.take(1), vec![i]);
		}
		assert!(buffer.is_empty());
	}

	#[test]
	fn test_alternating_extend_and_take() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();
		for i in 0..20 {
			buffer.extend(&[i]);
			assert_eq!(buffer.take(1), vec![i]);
			assert!(buffer.is_empty());
		}
	}

	#[test]
	fn test_get_raw() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();
		buffer.extend(&(0..7).collect::<Vec<i32>>());
		buffer.take(3);
		buffer.extend(&(7..12).collect::<Vec<i32>>());

		assert_eq!(buffer.raw_data(), &[3, 4, 5, 6, 7, 8, 9, 2]);
	}

	#[test]
	fn test_fill_drain_repeat() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();
		for round in 0..5 {
			let data:Vec<i32> = (0..TEST_CAPACITY as i32 - 1).map(|x| x + round * 10).collect();
			buffer.extend(&data);
			assert!(buffer.is_full());

			assert_eq!(buffer.take(TEST_CAPACITY), data);
			assert!(buffer.is_empty());
		}
	}

	#[test]
	fn test_is_full_and_is_empty_consistency() {
		let mut buffer:CircularBufferMultiread<i32, TEST_CAPACITY> = get_test_buffer();
		assert!(buffer.is_empty());
		assert!(!buffer.is_full());

		buffer.extend(&[1; TEST_CAPACITY - 1]);
		assert!(buffer.is_full());
		assert!(!buffer.is_empty());
	}

	#[test]
	fn test_stress_test_large_cycles() {
		const LOOPS:usize = 100_000;

		let mut buffer:CircularBufferMultiread<i32, 1024> = CircularBufferMultiread::new();
		let mut counter:i32 = 0;

		for _ in 0..LOOPS {
			let data:Vec<i32> = (0..512).map(|x| counter + x as i32).collect();
			buffer.extend(&data);
			counter += 512;

			assert_eq!(buffer.take(512).len(), 512);
		}
		assert!(buffer.is_empty());
	}

	#[test]
	fn test_performance_timing() {
		const OPERATIONS:usize = 1_000_000;

		let mut buffer:CircularBufferMultiread<i32, 2048> = CircularBufferMultiread::new();
		let mut data:Vec<i32> = vec![0i32; 1024];

		let start:Instant = Instant::now();
		for index in 0..OPERATIONS {
			for value in data.iter_mut() {
				*value = index as i32;
			}
			buffer.extend(&data);
			let _ = buffer.take(1024);
		}
		let elapsed:Duration = start.elapsed();
		println!("Performed {} ops in {:?}", OPERATIONS, elapsed);
	}
}