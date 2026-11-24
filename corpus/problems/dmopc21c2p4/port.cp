// Source: DMOPC 2021 Contest 2 P4 (Water Mechanics), https://dmoj.ca/problem/dmopc21c2p4
// Problem: N cells in a row with heights h_i <= 1e9 and an integer 1 <= K <= N <= 1e6. Water
//   poured into a cell spreads at most K cells each way over equal heights, the limit resets
//   whenever it flows down, and it never flows up. Print the fewest pours that wet every cell.
// Solution: group equal heights into runs; a run of length L alone needs ceil(L / (2K + 1))
//   pours. DP left to right over runs with a state for the boundary to the next run:
//   neutral, this run's water spills right into the lower next run (it then gets K + 1
//   cells free), or the higher next run's water spills left into this one (covering its
//   last K + 1 cells). O(N).
// original: corpus/submissions/3949962.cpp
// Group equal adjacent elements into (val, cnt) runs.
// States: 0 = Neutral, 1 = Push, 2 = Pull.

let n: 1..=1e6 = read()
let k: 1..=n = read()
let height: [n]0..=1e9 = read()

let stride = 2 * k + 1
fn pours(cells: Int) -> Int = if cells <= 0: 0 else: ceil_div(cells, stride)

// the fewest pours for the runs so far, by how the last run meets the next one
var alone: ?Int = 0      // on its own
var spills: ?Int = None  // its water also flows down into the next run
var pulled: ?Int = None  // the next run's water also flows down over its last K + 1 cells
var last: ?Int = None
for run in height.runs():
    let h = run[run.keys.first!]
    // this run's cells left to cover, after its left neighbour's choice
    var ways: List<(done: Int, cells: Int)> = []
    if let d = alone: ways.push((d, run.len))
    if let up = last and up > h:
        if let d = spills: ways.push((d, run.len - k - 1))
    elif let d = pulled:
        ways.push((d, run.len + 1))
    alone = min(w.done + pours(w.cells) for w in ways)
    spills = min(w.done + pours(w.cells + 1) for w in ways)
    pulled = min(w.done + pours(w.cells - k - 1) for w in ways)
    last = h
print(alone!)
