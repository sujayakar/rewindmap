use std::fmt::Debug;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum EntryType {
    Original {
        next_cascaded: Option<usize>,
    },
    Cascaded {
        prev_original: Option<usize>,
        next_level: usize,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Entry<T> {
    key: T,
    entry_type: EntryType,
}

#[derive(Debug)]
pub struct FractionalCascade<T> {
    levels: Vec<Vec<Entry<T>>>,
}

#[derive(Debug)]
enum CascadedPtr {
    Zero,
    Range {
        left_noninclusive: usize,
        right_inclusive: usize,
    },
    End,
}

impl<T: Copy + Clone + Debug + Ord> FractionalCascade<T> {
    pub fn new(items: Vec<Vec<T>>) -> Self {
        let mut items_iter = items.into_iter().rev();
        let last = items_iter.next().expect("No levels");
        let last_level: Vec<_> = last
            .into_iter()
            .map(|key| Entry { key, entry_type: EntryType::Original { next_cascaded: None } })
            .collect();

        let mut levels = vec![last_level];

        for items in items_iter {
            let mut level: Vec<_> = items
                .into_iter()
                .map(|key| Entry { key, entry_type: EntryType::Original { next_cascaded: None } }) // fix this up below
                .collect();

            for (ix, entry) in levels.last().unwrap().iter().enumerate().step_by(2) {
                let entry_type = EntryType::Cascaded {
                    prev_original: None, // fix this up below
                    next_level: ix,
                };
                level.push(Entry { key: entry.key, entry_type });
            }

            level.sort();

            let mut last_original = None;
            for (ix, item) in level.iter_mut().enumerate() {
                match item.entry_type {
                    EntryType::Original { .. } => {
                        last_original = Some(ix);
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

            levels.push(level);
        }

        levels.reverse();

        Self { levels }
    }

    // For each array, returns ix such that A[i] < key for all i < ix.
    pub fn bisect_left_naive(&self, key: T) -> Vec<usize> {
        let k = Entry { key, entry_type: EntryType::Original { next_cascaded: None } };
        self.levels.iter().map(|l| bisect_left(l, k)).collect()
    }

    pub fn bisect_left(&self, key: T) -> Vec<usize> {
        let mut out = vec![];

        let k = Entry { key, entry_type: EntryType::Original { next_cascaded: None } };
        let mut levels_iter = self.levels.iter();
        let first_level = levels_iter.next().unwrap();
        let start = bisect_left(first_level, k);

        let mut next_ix = if start >= first_level.len() {
            out.push(start);
            CascadedPtr::End
        } else {
            match first_level[start].entry_type {
                EntryType::Cascaded { prev_original, next_level } => {
                    out.push(prev_original.unwrap_or(0));
                    if next_level == 0 {
                        CascadedPtr::Zero
                    } else {
                        CascadedPtr::Range {
                            left_noninclusive: next_level - 2,
                            right_inclusive: next_level,
                        }
                    }
                },
                EntryType::Original { next_cascaded } => {
                    out.push(start);
                    match next_cascaded {
                        Some(ix) => {
                            match first_level[ix].entry_type {
                                EntryType::Cascaded { next_level, .. } => {
                                    if next_level == 0 {
                                        CascadedPtr::Zero
                                    } else {
                                        CascadedPtr::Range {
                                            left_noninclusive: next_level - 2,
                                            right_inclusive: next_level,
                                        }
                                    }
                                },
                                _ => panic!("Invalid cascaded ptr"),
                            }
                        }
                        None => CascadedPtr::End,
                    }
                },
            }
        };

        for level in levels_iter {
            let ix = match next_ix {
                CascadedPtr::Zero => 0,
                CascadedPtr::Range { left_noninclusive, right_inclusive } => {
                    assert_eq!(left_noninclusive + 2, right_inclusive);
                    if key <= level[left_noninclusive + 1].key {
                        left_noninclusive + 1
                    } else {
                        assert!(key <= level[right_inclusive].key);
                        right_inclusive
                    }
                },
                CascadedPtr::End => {
                    if level.len() % 2 == 0 {
                        if key <= level[level.len() - 1].key {
                            level.len() - 1
                        } else {
                            level.len()
                        }
                    } else {
                        level.len()
                    }
                },
            };
            next_ix = if ix >= level.len() {
                out.push(ix);
                CascadedPtr::End
            } else {
                match level[ix].entry_type {
                    EntryType::Cascaded { prev_original, next_level } => {
                        out.push(prev_original.unwrap_or(0));
                        if next_level == 0 {
                            CascadedPtr::Zero
                        } else {
                            CascadedPtr::Range {
                                left_noninclusive: next_level - 2,
                                right_inclusive: next_level,
                            }
                        }
                    },
                    EntryType::Original { next_cascaded } => {
                        out.push(ix);
                        match next_cascaded {
                            Some(ix) => {
                                match level[ix].entry_type {
                                    EntryType::Cascaded { next_level, .. } => {
                                        if next_level == 0 {
                                            CascadedPtr::Zero
                                        } else {
                                            CascadedPtr::Range {
                                                left_noninclusive: next_level - 2,
                                                right_inclusive: next_level,
                                            }
                                        }
                                    },
                                    _ => panic!("Invalid cascaded ptr"),
                                }
                            }
                            None => CascadedPtr::End,
                        }
                    }
                }
            };
        }

        out
    }
}

// Returns ix such that A[i] < key for all i < ix.
fn bisect_left<T: Ord>(array: &[T], key: T) -> usize {
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
            for e in &f.levels[i][0..ix] {
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
