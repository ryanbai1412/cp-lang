//! Path-hashed shards: a test over every file runs as `N` tests, each over
//! the files whose path hashes to its shard, so none outgrows the 30 s a
//! test may take. A file's shard depends on its path alone, so adding one
//! moves no other. Each sharded file also gets a test that its shards
//! cover every file once.

/// The shard of `N` that `key`, a file's path, falls in (FNV-1a, stable
/// across runs and machines).
pub fn shard_of(key: &str, n: usize) -> usize {
    let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    (hash % n as u64) as usize
}

/// The items of `items` in shard `k` of `n`, keyed by `key`.
pub fn mine<T>(items: Vec<T>, key: impl Fn(&T) -> String, k: usize, n: usize) -> Vec<T> {
    items
        .into_iter()
        .filter(|item| shard_of(&key(item), n) == k)
        .collect()
}

/// That `shards` are `0..n`, so every item is in exactly one of them.
pub fn assert_cover(keys: &[String], shards: &[usize], n: usize) {
    assert_eq!(
        shards,
        (0..n).collect::<Vec<_>>(),
        "one test per shard, in order"
    );
    for key in keys {
        assert!(shards.contains(&shard_of(key, n)), "{key} is in no shard");
    }
    assert!(!keys.is_empty(), "nothing to shard");
}

/// `N` tests named `$test = k` that each call `$run(k)`, and `$cover`,
/// checking that the shards cover every key of `$keys`.
macro_rules! shard_tests {
    ($run:ident, $keys:expr, $cover:ident, [$($test:ident = $k:literal),* $(,)?]) => {
        $(
            #[test]
            fn $test() {
                $run($k);
            }
        )*
        #[test]
        fn $cover() {
            let shards = [$($k),*];
            shards::assert_cover(&$keys, &shards, shards.len());
        }
    };
}
pub(crate) use shard_tests;
