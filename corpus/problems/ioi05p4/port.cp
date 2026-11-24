// Source: IOI 2005 Day 2 (Birthday), https://dmoj.ca/problem/ioi05p4
// Problem: n <= 1e6 children sit in order 1..n around a round table; reseat them so the circle
//   reads p_1..p_n in either direction, each child moving left or right. Print the minimum
//   possible largest distance any child moves.
// Solution: for one direction and a rotation s, the child put on seat i moves the circular
//   distance between s and its shift (i - p_i) mod n. Sort the n shifts: the best s is the
//   middle of the shortest arc holding them all, n minus the largest gap, so the answer is
//   (n - gap + 1) / 2. Take the better of p and its mirror. O(n log n).
// original: corpus/submissions/5044895.cpp
use std.rand.{Rng}

let n: 1..=1e6 = read()
domain Seat = 1..=n

fn random_wish() -> [Seat]Seat:
    var r = Rng(rand(0, 1e9))
    r.perm()

let wish: [Seat]Seat = read(gen: random_wish)

// With the circle turned by s, child i walks the distance between s and its
// own shift. The best turn is the middle of the arc that holds every shift.
fn worst(order: [Seat]Seat) -> Int:
    let shifts = [(i.label - order[i].label) % n for i in Seat].sorted()
    let wrap = shifts[0] + n - shifts.last!
    let gap = max(wrap, shifts.windows(2).map(w => w.last! - w.first!).max() ?? 0)
    (n - gap + 1) / 2

let mirrored = [Seat] i => wish[Seat(n + 1 - i.label)]
print(min(worst(wish), worst(mirrored)))
