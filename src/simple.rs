use std::fmt::Debug;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum EntryType {
    Original {
        next_cascaded: Option<usize>,
    },
    Cascaded {
        prev_original: usize,
        next_level: usize,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Entry<T> {
    key: T,
    entry_type: EntryType,
}

#[derive(Debug)]
struct Level<T> {
    entries: Vec<Entry<T>>,
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
            .map(|key| Entry { key, entry_type: EntryType::Original { next_cascaded: None } })
            .collect();

        let mut levels = vec![Level { entries: last_level }];

        for items in items_iter {
            let mut level: Vec<_> = items
                .into_iter()
                .map(|key| Entry { key, entry_type: EntryType::Original { next_cascaded: None } }) // fix this up below
                .collect();

            for (ix, entry) in levels.last().unwrap().entries.iter().enumerate().step_by(2) {
                let entry_type = EntryType::Cascaded {
                    prev_original: 0, // fix this up below
                    next_level: ix,
                };
                level.push(Entry { key: entry.key, entry_type });
            }

            level.sort();

            let mut last_original = 0;
            for (ix, item) in level.iter_mut().enumerate() {
                match item.entry_type {
                    EntryType::Original { .. } => {
                        last_original = ix;
                    },
                    EntryType::Cascaded { ref mut prev_original, .. } => {
                        *prev_original = last_original;
                    },
                }
            }

            let mut last_cascaded = None;
            for (ix, item) in level.iter_mut().enumerate().rev() {
                match item.entry_type {
                    EntryType::Original { ref mut next_cascaded } => {
                        *next_cascaded = last_cascaded;
                    },
                    EntryType::Cascaded { .. } => {
                        last_cascaded = Some(ix);
                    },
                }
            }

            levels.push(Level { entries: level });
        }

        levels.reverse();

        Self { levels }
    }

    // For each array, returns ix such that A[i] < key for all i < ix.
    pub fn bisect_left_naive(&self, key: T) -> Vec<usize> {
        let k = Entry { key, entry_type: EntryType::Original { next_cascaded: None } };
        self.levels.iter().map(|l| bisect_left(&l.entries, k)).collect()
    }

    #[inline(never)]
    fn cascade_ptr(level: &[Entry<T>], ix: usize) -> (usize, Option<usize>) {
        if ix >= level.len() {
            return (ix, None);
        }
        match level[ix].entry_type {
            EntryType::Cascaded { prev_original, next_level } => {
                (prev_original, Some(next_level))
            },
            EntryType::Original { next_cascaded: Some(cascaded_ix) } => {
                let next_level = match level[cascaded_ix].entry_type {
                    EntryType::Cascaded { next_level, .. } => next_level,
                    _ => panic!("Invalid cascaded ptr"),
                };
                (ix, Some(next_level))
            },
            EntryType::Original { next_cascaded: None } => (ix, None),
        }
    }

    #[inline(never)]
    pub fn bisect_left(&self, key: T) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.levels.len());
        let mut levels_iter = self.levels.iter();
        let first_level = match levels_iter.next() {
            Some(l) => l,
            None => return out,
        };
        let k = Entry { key, entry_type: EntryType::Original { next_cascaded: None } };
        let cur_ptr = bisect_left(&first_level.entries, k);
        let (result, mut next_ptr) = Self::cascade_ptr(&first_level.entries, cur_ptr);
        out.push(result);

        for level in levels_iter {
            let mut cur_ptr = next_ptr.unwrap_or(level.entries.len());
            let len = level.entries.len();

            // We know that the cascaded pointer has a value >= key in the previous level. We also
            // know that any previous element in the current array that was cascaded into the
            // previous level have a value < key. So, under which conditions do we need to move our
            // pointer back one position to get the correct result for `bisect_left`?
            // 1) The current pointer must not be zero.
            // 2) If the pointer is past the array, the array must have even length. If the array
            //    has odd length, the last element was cascaded, and we know it must be strictly
            //    less than `key`.
            // 3) The value at the `cur_ptr - 1` is greater than or equal to `key`.
            if cur_ptr != 0 && (cur_ptr != len || len % 2 == 0) && key <= level.entries[cur_ptr - 1].key {
                cur_ptr -= 1;
            }
            let (result, ptr) = Self::cascade_ptr(&level.entries, cur_ptr);
            next_ptr = ptr;
            out.push(result);
        }
        out
    }
}

// Returns ix such that A[i] < key for all i < ix.
#[inline(never)]
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
