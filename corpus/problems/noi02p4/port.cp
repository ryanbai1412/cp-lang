// Source: NOI 2002 P4 (Island of Cavemen), https://dmoj.ca/problem/noi02p4
// limits: 1000 ms, 16 MB
// Problem: 1 <= n <= 15 cavemen on a ring of m caves; caveman i starts in cave 1 <= c_i <= 100,
//   moves 1 <= p_i <= 100 caves a year and lives 0 <= l_i <= 1e6 years. Print the smallest m for
//   which no two ever share a cave while both live; it is at most 1e6.
// Solution: try m upward from the largest c_i. Two cavemen meet in year x when
//   (p_i - p_j) x = c_j - c_i (mod m); linear_congruence gives its least solution, and m fails
//   if that is at most both lifetimes. O(answer * n^2 log m).
// original: corpus/submissions/3741279.cpp
// Bounds: n: 1..=15
use std.math.{linear_congruence}

let n: 1..=15 = read()
struct Caveman:
    cave: 1..=100
    pace: 1..=100
    life: 0..=1e6
domain Man = 0..n
let men: [Man]Caveman = read()

fn meet(a: Caveman, b: Caveman, m: Int) -> Bool =
    linear_congruence(a.pace - b.pace, b.cave - a.cave, m).is_some_and(year => year.r <= min(a.life, b.life))

fn peaceful(m: Int) -> Bool = not any(meet(a, b, m) for (a, b) in men.pairs())

print((max(x.cave for x in men)!..=1_000_000).find(peaceful)!)
