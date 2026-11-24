// Source: SDOI 2009, School Canteen, https://dmoj.ca/problem/etsr23r2p4
// Problem: 1 <= C <= 5 tests. 1 <= n <= 1000 students queue; student i has taste
//   0 <= T_i <= 1000 and tolerance 0 <= B_i <= 7: at most the B_i people right behind i may be
//   served before i. Cooking a dish after one of taste a costs a xor b (the first is free).
//   Print the least total time.
// Solution: everyone before the first unserved student i is served, and nobody past i + 7 can
//   be, so a state is (i, the served set within the window i..=i+7, the last served student as
//   an offset from i in -8..8). From i unserved, student i + j may go next while j stays within
//   k + B_{i+k} for every unserved k < j; once i is served the window slides to i + 1. Sweep i
//   upward keeping two layers: O(n * 2^8 * 16 * 8) per test.
// original: corpus/submissions/5494510.cpp
@!multitest
let n: 1..=1000 = read()
struct Student:
    taste: 0..=1000
    patience: 0..=7
domain Pos = 0..n
let queue: [Pos]Student = read()
domain Offset = 0..8        // the window i, i + 1, ..., i + 7
domain Rel = -8..8          // the last served student, as an offset from i
type Window = Subset<Offset>

fn at(i: Pos, j: Int) -> Pos = Pos(i.ord + j)

// The students of the window who may be served next.
fn choices(i: Pos, served: Window) -> List<Offset>:
    var out = List<Offset>()
    var reach = Int(Offset.last!)
    for j in Offset:
        if Int(j) > reach or i.ord + Int(j) >= n: break
        if served.contains(j): continue
        out.push(j)
        reach = min(reach, Int(j) + queue[at(i, Int(j))].patience)
    out

// cur[served][last]: the least time for a state at the current i.
var cur: [Window][Rel]?Int = None
for j in choices(Pos.first!, Window.empty):
    cur[Window.single(j)][Rel(Int(j))] = 0      // the first dish is free
for i in Pos:
    var next: [Window][Rel]?Int = None
    for served in Window:                      // ascending: serving someone makes a larger set
        for last in Rel:
            if let time = cur[served][last]:
                if served.contains(Offset.first!):
                    let slid = last.prev!
                    next[served.shifted(-1)][slid] = min(next[served.shifted(-1)][slid] ?? time, time)
                else:
                    let before = queue[at(i, Int(last))].taste
                    for j in choices(i, served):
                        let cand = time + (before ^ queue[at(i, Int(j))].taste)
                        let to = served.with(j)
                        cur[to][Rel(Int(j))] = min(cur[to][Rel(Int(j))] ?? cand, cand)
    cur = next
print(cur[Window.empty].min()!)
