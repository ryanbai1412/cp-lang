// Source: IOI 2013 Day 2 P4, Cave, https://dmoj.ca/problem/ioi13p4
// Problem: interactive: N switches (N <= 5000), each up or down, each wired to a different
//   door of N doors in a row; a door is open iff its switch is in its hidden correct position.
//   tryCombination(S) returns the first closed door (or -1); within 70 000 calls, find every
//   switch's correct position and its door, then call answer(S, D).
// Solution: Settle doors 0, 1, ... in order, holding settled switches in their open position.
//   For door d, one query with every free switch up shows which position opens d; then binary
//   search on k, setting the first k free switches to that position and the rest to the other,
//   for the smallest k that opens d: its k-th free switch is d's. At most 1 + 13 queries per
//   door, O(N^2 log N) time.
// The task is a grader function and the judge supplies `try_combination`; this plays the sample
// grader: it reads N, each switch's open position and its door, answers each try the way the
// judge does, and prints `Accepted: q`. The `protocol` form is the dream in
// research/frictions/ioi13p4.md.
// original: corpus/submissions/4687973.cpp
let n: 1..=5000 = read()
domain Switch = 0..n
domain Door = 0..n
let hidden_open: [Switch]0..=1 = read()
let hidden_door: [Switch]Door = read()

var tries = 0
fn try_combination(s: [Switch]0..=1) -> ?Door:
    tries += 1
    assert tries <= 70_000
    [hidden_door[w] for w in Switch if s[w] != hidden_open[w]].min()

fn explore_cave() -> ([Switch]0..=1, [Switch]Door):
    var open: [Switch]0..=1 = 0
    var door: [Switch]?Door = None
    for d in Door:
        // Settled switches hold their doors open; `free` are the rest.
        let free = [w for w in Switch if door[w] == None]
        fn opens(k: Int, to: 0..=1) -> Bool:
            var s = [Switch] w => if door[w] != None: open[w] else: 1 - to
            for w in free[..k]: s[w] = to
            if let first_closed = try_combination(s): first_closed > d else: true
        // Which position opens door d, then which free switch carries it.
        let to = if opens(free.len, 0): 0 else: 1
        let k = (1..=free.len).first_true(k => opens(k, to))!
        open[free[k - 1]] = to
        door[free[k - 1]] = Some(d)
    (open, [Switch] w => door[w]!)

let (found_open, found_door) = explore_cave()
assert found_open == hidden_open and found_door == hidden_door
print("Accepted:", tries)
