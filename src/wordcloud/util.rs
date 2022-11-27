pub fn next_multiple(x: usize, m: usize) -> usize {
	// https://math.stackexchange.com/q/291468/682201
	((x-1)|(m-1)) + 1
}

// Little Endian - left most bit is smallest
pub fn bits(v: usize) -> [usize; usize::BITS as usize] {
	let mut res = [0; usize::BITS as usize];
	for i in 0..(usize::BITS as usize) {
		res[i] = (v >> i) & 1;
		assert!(res[i] <= 1);
	}
	res
}

fn unbits(bits: [usize; usize::BITS as usize]) -> usize {
	return bits.into_iter().rev().fold(0, |acc, curr| (acc << 1) + curr);
}

mod tests {
	use super::*;

	fn test_roundtrip(n: usize) {
		assert_eq!(unbits(bits(n)), n)
	}

	#[test]
	fn test_bits() {
		test_roundtrip(0);
		test_roundtrip(1);
		test_roundtrip(1532);
	}
}