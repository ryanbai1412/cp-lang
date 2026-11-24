//! Programs whose run time must grow with the input as their cost says,
//! not faster: each one was once quadratic.

use super::*;

#[test]
fn user_ordered_set_scales_as_n_log_n() {
    // A set over a user `Ord` key keeps it in sorted runs; one sorted list, shifted whole on each
    // insert, took egoi23p2 144 s at n = 200000.
    let src = "\
let n = read<Int>()
var s = Set<(Int, Desc<Int>)>()
for i in 0..n: s.insert((i % 7, desc(i)))
for i in 0..n: s.remove((i % 7, desc(i)))
print(s.len)
";
    let time = |n: usize| {
        let started = std::time::Instant::now();
        let (out, trap) = outcome(src, &format!("{n}\n"));
        assert!(trap.is_none(), "unexpected trap: {trap:?}");
        assert_eq!(out, "0\n");
        started.elapsed().as_secs_f64()
    };
    let (small, big) = (time(5_000), time(40_000));
    assert!(
        big < 20.0 * small,
        "8x the keys took {:.1}x the time",
        big / small
    );
}

#[test]
fn slicing_a_table_once_per_run_stays_linear() {
    let n: i64 = 60_000;
    let vals: Vec<String> = (0..n).map(|i| i.to_string()).collect();
    let input = format!("{n}\n{}\n", vals.join(" "));
    let source = "\
let n: 1..=100000 = read()
let xs: [0..n]0..=1000000 = read()
var count = 0
var keys = 0
for run in xs.runs():
    count += 1
    keys += run.keys.first!
let mid = xs[2..5]
print(count, keys, mid.len, mid.first!, mid.keys.first!)
";
    let (out, failure) = outcome(source, &input);
    assert_eq!(failure, None);
    assert_eq!(out, format!("{n} {} 3 2 2\n", n * (n - 1) / 2));
}
