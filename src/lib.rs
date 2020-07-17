#![feature(generator_trait)]
#![feature(generators)]

// mod cascade;
pub mod simple;

// use std::collections::BTreeMap;
// use std::ops::RangeBounds;
// use std::marker::Unpin;
// use std::ops::{
//     Generator,
//     GeneratorState,
// };
// use std::pin::Pin;

// pub struct GenIterator<T> {
//     pub inner: T,
// }

// impl<T: Generator<Return = ()> + Unpin> Iterator for GenIterator<T> {
//     type Item = T::Yield;

//     fn next(&mut self) -> Option<Self::Item> {
//         match Pin::new(&mut self.inner).resume(()) {
//             GeneratorState::Yielded(item) => Some(item),
//             GeneratorState::Complete(()) => None,
//         }
//     }
// }

// macro_rules! iterator {
//     ($e:expr) => {{
//         $crate::GenIterator {
//             inner: move || {
//                 #[allow(unreachable_code)]
//                 {
//                     if false {
//                         yield unreachable!();
//                     }
//                 }
//                 $e
//             },
//         }
//     }};
// }

// type Timestamp = u64;
// type KeyRank = u32;
// type ValueRank = u32;

// struct KeyList {
//     history: Vec<(Timestamp, ValueRank)>,
// }

// impl KeyList {
//     fn new() -> Self {
//         Self {
//             history: vec![],
//         }
//     }
// }

// struct KeyListCascade {
//     keys: Vec<KeyList>,
// }

// impl KeyListCascade {
//     fn new() -> Self {
//         Self {
//             keys: vec![],
//         }
//     }
// }

// pub struct RewindMap<K, V> {
//     keys: BTreeMap<K, KeyRank>,
//     key_lists: KeyListCascade,
//     values: Vec<V>,
// }

// impl<K: Eq + Ord, V> RewindMap<K, V> {
//     pub fn new(tuples: impl Iterator<Item = (K, Timestamp, Option<V>)>) -> Self {
//         let mut triples = BTreeMap::new();

//         for (k, ts, v) in tuples {
//             triples.entry(k).or_insert_with(Vec::new).push((ts, v));
//         }

//         let mut keys = BTreeMap::new();
//         let mut key_lists = KeyListCascade::new();
//         let mut values = vec![];

//         for (k, mut pairs) in triples {
//             let key_rank = key_lists.keys.len() as u32;
//             keys.insert(k, key_rank);

//             let mut key_list = KeyList::new();
//             pairs.sort_by_key(|&(ts, _)| ts);
//             for (ts, v) in pairs {
//                 let value_rank = match v {
//                     Some(v) => {
//                         let rank = values.len() as u32;
//                         values.push(v);
//                         rank
//                     },
//                     None => std::u32::MAX,
//                 };
//                 key_list.history.push((ts, value_rank));
//             }
//             key_lists.keys.push(key_list);
//         }

//         Self { keys, key_lists, values }
//     }

//     pub fn history<'a>(&'a self, key: &'a K, ts_range: impl RangeBounds<Timestamp>) -> impl Iterator<Item = (Timestamp, Option<&'a V>)> + 'a {
//         iterator!({
//             let key_list = match self.keys.get(key) {
//                 Some(&key_rank) => &self.key_lists.keys[key_rank as usize],
//                 None => return,
//             };
//             for &(ts, value_rank) in &key_list.history {
//                 let value = if value_rank == std::u32::MAX {
//                     None
//                 } else {
//                     Some(&self.values[value_rank as usize])
//                 };
//                 yield (ts, value);
//             }
//         })
//     }

//     pub fn get(&self, ts: Timestamp, key: &K) -> Option<&V> {
//         unimplemented!();
//     }

//     pub fn range<'a>(&'a self, ts: Timestamp, key_range: impl RangeBounds<K> + 'a) -> impl Iterator<Item = (&'a K, &'a V)> + 'a {
//         iterator!({
//             for (key, &key_rank) in self.keys.range(key_range) {
//                 let key_list = &self.key_lists.keys[key_rank as usize];
//             }
//         })
//     }
// }

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
