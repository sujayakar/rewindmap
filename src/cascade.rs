use rsdict::RsDict;
use std::slice::SliceIndex;

struct Level {
    keys: Vec<u32>,
    is_cascade: RsDict,
}

impl Level {
    // Given an element x, return ix such that...
    // all(val < x for val in self.keys[..ix])
    // all(x <= val for val in self.keys[ix..])
    fn bisect_left(&self, x: u32) -> usize {

        // Invariant: x < keys[i] for all i < lo
        // Invariant: keys[i] <= x for all hi <= i
        let mut lo = 0;
        let mut hi = self.keys.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if unsafe { *self.keys.get_unchecked(mid) } < x {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }
}

struct FractionalCascade {
    levels: Vec<Level>,
}

const CASCADE_FRACTION: usize = 2;

impl FractionalCascade {
    fn new(levels: &[&[u32]]) -> Self {
        let mut levels_rev = levels.iter().rev();

        let last_level = match levels_rev.next() {
            None => return Self { levels: vec![] },
            Some(l) => l.to_vec(),
        };
        let mut is_cascade = RsDict::with_capacity(last_level.len());
        for _ in 0..last_level.len() {
            is_cascade.push(false);
        }

        let mut levels = Vec::with_capacity(levels.len());
        let mut last_level = Level { keys: last_level, is_cascade };

        for level in levels_rev {
            let mut level: Vec<_> = level.iter().map(|i| (i, false)).collect();
            for (i, elem) in last_level.keys.iter().enumerate() {
                if i % CASCADE_FRACTION == 0 {
                    level.push((elem, true));
                }
            }
            level.sort();

            let mut new_level = Level {
                keys: Vec::with_capacity(level.len()),
                is_cascade: RsDict::with_capacity(level.len()),
            };

            for (&key, is_cascade) in level {
                new_level.keys.push(key);
                new_level.is_cascade.push(is_cascade);
            }

            levels.push(last_level);
            last_level = new_level;
        }

        levels.push(last_level);
        levels.reverse();

        Self { levels }
    }

    // Index of greatest element <= key
    fn predecessor(&self, key: u32, level_start: usize, level_end: usize) -> Vec<usize> {
        let mut levels_iter = self.levels[level_start..level_end].iter();

        let first_level = match levels_iter.next() {
            Some(l) => l,
            None => return vec![],
        };

        let mut out = Vec::with_capacity(level_end - level_start);

        let b = first_level.bisect_left(key);
        let (is_cascade, cascade_rank) = first_level.is_cascade.bit_and_one_rank(b as u64);

        let (output_ix, next_ix) = if is_cascade {
            // Note that we're allowed to have duplicate timestamps if a cascaded timestamp matches
            // one from our list. The original values sort first, so we just need to find the previous
            // original value.
            let num_original = b as u64 - cascade_rank;

            // If there's no original values to our left, just emit zero.
            let output_ix = if num_original == 0 {
                0
            } else {
                // If there's k values to our left, select the last one (zero indexed)
                first_level.is_cascade.select0(num_original - 1)
                    .expect("Failed to find original value to the left")
            };

            // We know that all values ot the left of this are less than `key` in the current level,
            // so the position in the next level must exist in
            // [cascade_rank * CASCADE_FRACTION, (cascade_rank + 1) * CASCADE_FRACTION)
            let next_ix = cascade_rank as usize * CASCADE_FRACTION;

            (output_ix, next_ix)
        } else {



            (b, next_ix)
        }



        let first_ix = match first_level.keys.binary_search(&key) {
            Ok(s) => s,
            Err(s) => s,
        };

        let (is_cascade, cascade_rank) = first_level.is_cascade.bit_and_one_rank(first_ix as u64);


        let (output_ix, next_ix) = if is_cascade {


            let output_ix = first_level.is_cascade.select0(
        }


        if is_cascade {
            let prev_original = first_level.is_cascade.select0(first_ix as u64 - cascade_rank)
                // If there's no original values to the left of us, bottom out at index zero.
                .unwrap_or(0);
            out.push(prev_original);

            let next_ix = cascade_rank * CASCADE_FRACTION;
        } else {
            out.push(first_ix);
            let next_ix = first_level.is_cascade.select1(cascade_rank as u64).unwrap();

        }





        vec![]
    }

}
