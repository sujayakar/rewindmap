use std::fmt::Debug;
use std::usize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Entry<T> {
    key: T,
    is_cascaded: bool,

    // Pointer to the preceding original element if cascaded, current index if original.
    prev_original: usize,

    // Pointer to next cascaded element's index in next level if original, index itself if cascaded.
    next_level: usize,
    check_preceding: bool,
}

#[derive(Debug)]
struct Level<T> {
    entries: Vec<Entry<T>>,
    next_len: usize,
    check_preceding: bool,
}

#[derive(Debug)]
pub struct FractionalCascade<T> {
    levels: Vec<Level<T>>,
}

impl<T: Copy + Clone + Debug + Ord> FractionalCascade<T> {
    pub fn new(items: Vec<Vec<T>>) -> Self {
        let mut items_iter = items.into_iter().rev();
        let last = items_iter.next().expect("No levels");
        let last_level: Vec<_> = last
            .into_iter()
            .enumerate()
            .map(|(i, key)| Entry { key, is_cascaded: false, prev_original: i, next_level: usize::MAX, check_preceding: false })
            .collect();

        let mut levels = vec![Level { entries: last_level, next_len: usize::MAX, check_preceding: false }];

        for items in items_iter {
            let mut level: Vec<_> = items
                .into_iter()
                .enumerate()
                // Fix up the `next_level` below.None
                .map(|(i, key)| Entry { key, is_cascaded: false, prev_original: i, next_level: usize::MAX, check_preceding: false })
                .collect();

            let prev_level = levels.last().unwrap();
            let prev_len = prev_level.entries.len();
            for (i, entry) in prev_level.entries.iter().enumerate().step_by(2) {
                // Fix up `prev_original` below
                level.push(Entry { key: entry.key, is_cascaded: true, prev_original: usize::MAX, next_level: i, check_preceding: false });
            }

            level.sort();

            // Fix up the `prev_original` field for cascaded entries.
            let mut last_original = 0;
            for (i, entry) in level.iter_mut().enumerate() {
                if entry.is_cascaded {
                    entry.prev_original = last_original;
                } else {
                    last_original = i;
                }
            }

            // Fix up the `next_level` for original entries.
            let mut next_level = prev_len;
            for entry in level.iter_mut().rev() {
                if entry.is_cascaded {
                    next_level = entry.next_level;
                } else {
                    entry.next_level = next_level;
                }
                entry.check_preceding = entry.next_level != 0
                    && (entry.next_level != prev_len || prev_len % 2 == 0);
            }
            let check_preceding = prev_len != 0 && prev_len % 2 == 0;
            levels.push(Level { entries: level, next_len: prev_len, check_preceding });
        }

        levels.reverse();

        Self { levels }
    }

    // For each array, returns ix such that A[i] < key for all i < ix.
    pub fn bisect_left_naive(&self, key: T) -> Vec<usize> {
        let k = Entry { key, is_cascaded: false, prev_original: 0, next_level: 0, check_preceding: false };
        self.levels.iter().map(|l| bisect_left(&l.entries, k)).collect()
    }

    fn cascade_ptr(level: &Level<T>, ix: usize) -> (usize, usize, bool) {
        if ix >= level.entries.len() {
            return (ix, level.next_len, level.check_preceding);
        }
        let entry = unsafe { &*level.entries.get_unchecked(ix) };
        (entry.prev_original, entry.next_level, entry.check_preceding)
    }

    pub fn bisect_left(&self, key: T) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.levels.len());
        let mut levels_iter = self.levels.iter();
        let first_level = match levels_iter.next() {
            Some(l) => l,
            None => return out,
        };
        let k = Entry { key, is_cascaded: false, prev_original: 0, next_level: 0, check_preceding: false };
        let cur_ptr = bisect_left(&first_level.entries, k);
        let (result, mut next_ptr, mut check_preceding) = Self::cascade_ptr(&first_level, cur_ptr);
        out.push(result);

        for level in levels_iter {
            let mut cur_ptr = next_ptr;//.unwrap_or(level.entries.len());

            // We know that the cascaded pointer has a value >= key in the previous level. We also
            // know that any previous element in the current array that was cascaded into the
            // previous level have a value < key. So, under which conditions do we need to move our
            // pointer back one position to get the correct result for `bisect_left`?
            // 1) The current pointer must not be zero.
            // 2) If the pointer is past the array, the array must have even length. If the array
            //    has odd length, the last element was cascaded, and we know it must be strictly
            //    less than `key`.
            // 3) The value at the `cur_ptr - 1` is greater than or equal to `key`.
            if check_preceding && key <= level.entries[cur_ptr - 1].key {
                cur_ptr -= 1;
            }
            let (result, ptr, cp) = Self::cascade_ptr(&level, cur_ptr);
            next_ptr = ptr;
            check_preceding = cp;
            out.push(result);
        }
        out
    }
}

// Returns ix such that A[i] < key for all i < ix.
pub fn bisect_left<T: Ord>(array: &[T], key: T) -> usize {
    let mut lo = 0;
    let mut hi = array.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if unsafe { array.get_unchecked(mid) } < &key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

#[cfg(test)]
mod tests {
    use super::{FractionalCascade, bisect_left};
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_array_bisect_left() {
        let v = vec![1, 2, 3];
        assert_eq!(bisect_left(&v, 4), 3);
    }

    #[quickcheck]
    fn fc_bisect_left_invariants(mut array: Vec<Vec<u8>>, key: u8) {
        if array.is_empty() || array.iter().any(|a| a.is_empty()) {
            return;
        }
        for a in &mut array {
            a.sort();
        }
        let f = FractionalCascade::new(array);
        for (i, ix) in f.bisect_left(key).into_iter().enumerate() {
            for e in &f.levels[i].entries[0..ix] {
                assert!(e.key < key);
            }
        }
    }

    #[quickcheck]
    fn array_bisect_left_invariants(mut array: Vec<u8>, key: u8) {
        array.sort();
        let ix = bisect_left(&array, key);

        for i in 0..ix {
            assert!(array[i] < key);
        }
        for i in ix..array.len() {
            assert!(key <= array[i]);
        }
    }
}
