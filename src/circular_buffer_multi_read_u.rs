#[cfg(test)]
mod tests {
	use crate::{ CircularBufferMultiRead, ReadCursor };
	use std::time::{ Duration, Instant };
	
	

	const TEST_CAPACITY:usize = 8;
	const TEST_MAX_CURSOR_COUNT:usize = 12;
	fn get_test_buffer() -> CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> {
		CircularBufferMultiRead::new()
	}



	/* BASIC CIRCULAR BUFFER TESTS */

	#[test]
	fn test_new_buffer_is_empty() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		assert_eq!(buffer.len(&cursor), 0);
		assert!(buffer.is_empty(&cursor));
		assert!(!buffer.is_full(&cursor));
	}

	#[test]
	fn test_extend_and_take_simple() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		// Test write.
		let written:usize = buffer.extend(&[1, 2, 3]);
		assert_eq!(written, 3);
		assert_eq!(buffer.len(&cursor), 3);

		// Test take.
		assert_eq!(buffer.take(3, &cursor), vec![1, 2, 3]);
		assert!(buffer.is_empty(&cursor));
	}

	#[test]
	fn test_extend_over_capacity_truncates() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		let written:usize = buffer.extend(&(0..20).collect::<Vec<i32>>());
		assert_eq!(written, TEST_CAPACITY - 1); // Should always keep one "empty" slot. This makes sure both cursors with the same value always means the buffer is empty, rather than full.
		assert!(buffer.is_full(&cursor));
		assert_eq!(buffer.len(&cursor), TEST_CAPACITY - 1);
	}

	#[test]
	fn test_take_more_than_available() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		buffer.extend(&[1, 2, 3]);
		let taken_data:Vec<i32> = buffer.take(10, &cursor);
		assert_eq!(taken_data, vec![1, 2, 3]);
		assert!(buffer.is_empty(&cursor));
	}

	#[test]
	fn test_wraparound_behavior() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		// First take.
		let written:usize = buffer.extend(&[1, 2, 3, 4, 5, 6, 7]);
		assert_eq!(written, 7);
		assert!(buffer.is_full(&cursor));
		assert_eq!(buffer.take(4, &cursor), vec![1, 2, 3, 4]);
		assert_eq!(buffer.len(&cursor), 3);
		
		// Second take.
		let written:usize = buffer.extend(&[8, 9, 10]);
		assert_eq!(buffer.len(&cursor), 6);
		assert_eq!(written, 3);
		assert_eq!(buffer.take(6, &cursor), vec![5, 6, 7, 8, 9, 10]);
		assert!(buffer.is_empty(&cursor));
	}

	#[test]
	fn test_multiple_small_writes_and_reads() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		for i in 0..5 {
			assert_eq!(buffer.extend(&[i]), 1);
		}
		assert_eq!(buffer.len(&cursor), 5);

		for i in 0..5 {
			assert_eq!(buffer.take(1, &cursor), vec![i]);
		}
		assert!(buffer.is_empty(&cursor));
	}

	#[test]
	fn test_alternating_extend_and_take() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		for i in 0..20 {
			buffer.extend(&[i]);
			assert_eq!(buffer.take(1, &cursor), vec![i]);
			assert!(buffer.is_empty(&cursor));
		}
	}

	#[test]
	fn test_get_raw() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		buffer.extend(&(0..7).collect::<Vec<i32>>());
		buffer.take(3, &cursor);
		buffer.extend(&(7..12).collect::<Vec<i32>>());

		assert_eq!(buffer.raw_data(&cursor), &[3, 4, 5, 6, 7, 8, 9, 2]);
	}

	#[test]
	fn test_fill_drain_repeat() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		for round in 0..5 {
			let data:Vec<i32> = (0..TEST_CAPACITY as i32 - 1).map(|x| x + round * 10).collect();
			buffer.extend(&data);
			assert!(buffer.is_full(&cursor));

			assert_eq!(buffer.take(TEST_CAPACITY, &cursor), data);
			assert!(buffer.is_empty(&cursor));
		}
	}

	#[test]
	fn test_is_full_and_is_empty_consistency() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor:ReadCursor = buffer.create_read_cursor();

		assert!(buffer.is_empty(&cursor));
		assert!(!buffer.is_full(&cursor));

		buffer.extend(&[1; TEST_CAPACITY - 1]);
		assert!(buffer.is_full(&cursor));
		assert!(!buffer.is_empty(&cursor));
	}

	#[test]
	fn test_brute_force_cursors() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = CircularBufferMultiRead::new();
		let cursor:ReadCursor = buffer.create_read_cursor();
		
		const DATA_MAX:usize = TEST_CAPACITY + 6;
		let writable_data:Vec<i32> = (0..DATA_MAX).map(|index| index as i32).collect();
		for write_data_size in 0..DATA_MAX {
			for read_data_size in 0..DATA_MAX {
				println!("write size: {write_data_size}\tread size: {read_data_size}");
				buffer.extend(&writable_data[..write_data_size]);
				let taken_data:Vec<i32> = buffer.take(read_data_size, &cursor);

				let expected_data_size:usize = write_data_size.min(read_data_size).min(TEST_CAPACITY - 1);
				assert_eq!(taken_data, writable_data[..expected_data_size]);

				buffer.take_all(&cursor);
			}
		}
	}



	/* MULTI-READ CURSOR CIRCULAR BUFFER TESTS */

	#[test]
	fn test_multi_read_extend_and_take_simple() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		// Test write.
		let written:usize = buffer.extend(&[1, 2, 3]);
		assert_eq!(written, 3);
		assert_eq!(buffer.len(&cursor_a), 3);
		assert_eq!(buffer.len(&cursor_b), 3);

		// Test take.
		assert_eq!(buffer.take(3, &cursor_a), vec![1, 2, 3]);
		assert_eq!(buffer.take(3, &cursor_b), vec![1, 2, 3]);
		assert!(buffer.is_empty(&cursor_a));
		assert!(buffer.is_empty(&cursor_b));
	}

	#[test]
	fn test_multi_read_take_more_than_available() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		buffer.extend(&[1, 2, 3]);
		assert_eq!(buffer.take(10, &cursor_a), vec![1, 2, 3]);
		assert_eq!(buffer.take(10, &cursor_b), vec![1, 2, 3]);
		assert!(buffer.is_empty(&cursor_a));
		assert!(buffer.is_empty(&cursor_b));
	}

	#[test]
	fn test_multi_read_wraparound_behavior() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		// First take.
		let written:usize = buffer.extend(&[1, 2, 3, 4, 5, 6, 7]);
		assert_eq!(written, 7);
		assert!(buffer.is_full(&cursor_a));
		assert!(buffer.is_full(&cursor_b));
		assert_eq!(buffer.take(4, &cursor_a), vec![1, 2, 3, 4]);
		assert_eq!(buffer.take(4, &cursor_b), vec![1, 2, 3, 4]);
		assert_eq!(buffer.len(&cursor_a), 3);
		assert_eq!(buffer.len(&cursor_b), 3);
		
		// Second take.
		let written:usize = buffer.extend(&[8, 9, 10]);
		assert_eq!(buffer.len(&cursor_a), 6);
		assert_eq!(buffer.len(&cursor_b), 6);
		assert_eq!(written, 3);
		assert_eq!(buffer.take(6, &cursor_a), vec![5, 6, 7, 8, 9, 10]);
		assert_eq!(buffer.take(6, &cursor_b), vec![5, 6, 7, 8, 9, 10]);
		assert!(buffer.is_empty(&cursor_a));
		assert!(buffer.is_empty(&cursor_b));
	}

	#[test]
	fn test_multi_read_multiple_small_writes_and_reads() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		for i in 0..5 {
			assert_eq!(buffer.extend(&[i]), 1);
		}
		assert_eq!(buffer.len(&cursor_a), 5);

		for i in 0..5 {
			assert_eq!(buffer.take(1, &cursor_a), vec![i]);
			assert_eq!(buffer.take(1, &cursor_b), vec![i]);
		}
		assert!(buffer.is_empty(&cursor_a));
		assert!(buffer.is_empty(&cursor_b));
	}

	#[test]
	fn test_multi_read_alternating_extend_and_take() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		for i in 0..20 {
			buffer.extend(&[i]);
			assert_eq!(buffer.take(1, &cursor_a), vec![i]);
			assert_eq!(buffer.take(1, &cursor_b), vec![i]);
			assert!(buffer.is_empty(&cursor_a));
			assert!(buffer.is_empty(&cursor_b));
		}
	}

	#[test]
	fn test_multi_read_get_raw() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		buffer.extend(&(0..7).collect::<Vec<i32>>());
		buffer.take(3, &cursor_a);
		buffer.take(3, &cursor_b);
		buffer.extend(&(7..12).collect::<Vec<i32>>());

		assert_eq!(buffer.raw_data(&cursor_a), &[3, 4, 5, 6, 7, 8, 9, 2]);
		assert_eq!(buffer.raw_data(&cursor_b), &[3, 4, 5, 6, 7, 8, 9, 2]);
	}

	#[test]
	fn test_multi_read_fill_drain_repeat() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		for round in 0..5 {
			let data:Vec<i32> = (0..TEST_CAPACITY as i32 - 1).map(|x| x + round * 10).collect();
			buffer.extend(&data);
			assert!(buffer.is_full(&cursor_a));
			assert!(buffer.is_full(&cursor_b));

			assert_eq!(buffer.take(TEST_CAPACITY, &cursor_a), data);
			assert_eq!(buffer.take(TEST_CAPACITY, &cursor_b), data);
			assert!(buffer.is_empty(&cursor_a));
			assert!(buffer.is_empty(&cursor_b));
		}
	}

	#[test]
	fn test_multi_read_is_full_and_is_empty_consistency() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = get_test_buffer();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		assert!(buffer.is_empty(&cursor_a));
		assert!(buffer.is_empty(&cursor_b));
		assert!(!buffer.is_full(&cursor_a));
		assert!(!buffer.is_full(&cursor_b));

		buffer.extend(&[1; TEST_CAPACITY - 1]);
		assert!(buffer.is_full(&cursor_a));
		assert!(buffer.is_full(&cursor_b));
		assert!(!buffer.is_empty(&cursor_a));
		assert!(!buffer.is_empty(&cursor_b));
	}

	#[test]
	fn test_multi_read_stress_test_large_cycles() {
		const LOOPS:usize = 100_000;

		let mut buffer:CircularBufferMultiRead<i32, 1024, TEST_MAX_CURSOR_COUNT> = CircularBufferMultiRead::new();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		let mut counter:i32 = 0;
		for _ in 0..LOOPS {
			let data:Vec<i32> = (0..512).map(|x| counter + x as i32).collect();
			buffer.extend(&data);
			counter += 512;

			assert_eq!(buffer.take(512, &cursor_a).len(), 512);
			assert_eq!(buffer.take(512, &cursor_b).len(), 512);
		}
		assert!(buffer.is_empty(&cursor_a));
		assert!(buffer.is_empty(&cursor_b));
	}

	#[test]
	fn test_multi_read_performance_timing() {
		const OPERATIONS:usize = 1_000_000;

		let mut buffer:CircularBufferMultiRead<i32, 2048, TEST_MAX_CURSOR_COUNT> = CircularBufferMultiRead::new();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		let mut data:Vec<i32> = vec![0i32; 1024];
		let start:Instant = Instant::now();
		for index in 0..OPERATIONS {
			for value in data.iter_mut() {
				*value = index as i32;
			}
			buffer.extend(&data);
			let _ = buffer.take(1024, &cursor_a);
			let _ = buffer.take(1024, &cursor_b);
		}
		let elapsed:Duration = start.elapsed();
		println!("Performed {} ops in {:?}", OPERATIONS, elapsed);
	}

	#[test]
	fn test_multi_read_complex_read_altering() {
		let mut buffer:CircularBufferMultiRead<i32, 350, TEST_MAX_CURSOR_COUNT> = CircularBufferMultiRead::new();
		let cursors:Vec<ReadCursor> = (0..8).map(|_| buffer.create_read_cursor()).collect();
		
		// Write semi-randomized batch sizes of data.
		let write_data:Vec<i32> = (0..1000).map(|index| index).collect();
		let mut read_data:[Vec<i32>; 8] = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
		let mut write_data_cursor:usize = 0;
		for write_batch_size in [60, 40, 160, 140, 100, 200, 180, 120] {
			buffer.extend(&write_data[write_data_cursor..write_data_cursor + write_batch_size]);
			write_data_cursor += write_batch_size;

			// Read semi-randomized batch sizes of data.
			for (cursor_index, cursor) in cursors.iter().enumerate() {
				let read_batch_size:usize = write_batch_size - cursor_index * 3;
				read_data[cursor_index].extend(buffer.take(read_batch_size, cursor));
			}
		}
		for (cursor_index, cursor) in cursors.iter().enumerate() {
			read_data[cursor_index].extend(buffer.take_all(cursor));
		}

		// Validate all data was captured.
		for result_data in read_data {
			assert_eq!(result_data, write_data);
		}
	}

	#[test]
	fn test_multi_read_brute_force_cursors() {	
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = CircularBufferMultiRead::new();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		const DATA_MAX:usize = TEST_CAPACITY + 6;
		let writable_data:Vec<i32> = (0..DATA_MAX).map(|index| index as i32).collect();
		for write_data_size in 0..DATA_MAX {
			for read_data_size_a in 0..DATA_MAX {
				for read_data_size_b in 0..DATA_MAX {
					println!("write size: {write_data_size}\tread size a: {read_data_size_a}\tread size b: {read_data_size_b}");
					buffer.extend(&writable_data[..write_data_size]);
					let taken_data_a:Vec<i32> = buffer.take(read_data_size_a, &cursor_a);
					let taken_data_b:Vec<i32> = buffer.take(read_data_size_b, &cursor_b);

					let expected_data_size_a:usize = write_data_size.min(read_data_size_a).min(TEST_CAPACITY - 1);
					let expected_data_size_b:usize = write_data_size.min(read_data_size_b).min(TEST_CAPACITY - 1);
					assert_eq!(taken_data_a, writable_data[..expected_data_size_a]);
					assert_eq!(taken_data_b, writable_data[..expected_data_size_b]);

					buffer.take_all(&cursor_a);
					buffer.take_all(&cursor_b);
				}
			}
		}
	}

	#[test]
	fn test_multi_read_skip_cursor_without_take() {
		let mut buffer:CircularBufferMultiRead<i32, TEST_CAPACITY, TEST_MAX_CURSOR_COUNT> = CircularBufferMultiRead::new();
		let cursor_a:ReadCursor = buffer.create_read_cursor();
		let cursor_b:ReadCursor = buffer.create_read_cursor();

		let data:Vec<i32> = (0..6).collect();
		for _ in 0..8 {
			buffer.extend(&data);
			assert_eq!(buffer.take(8, &cursor_a), data);
			buffer.skip_current_data(&cursor_b);
		}
	}
}