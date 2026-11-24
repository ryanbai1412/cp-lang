// Source: IOI 2012 Day 2 (Last Supper), https://dmoj.ca/problem/ioi12p5
// Problem: N <= 1e5 colours, a scaffold holding K < N of them (initially 0..K-1) and N
//   colour requests. Program 1 sees all requests and writes at most N + K advice bits;
//   program 2 gets the bits and the requests one at a time, and on each miss must choose
//   which colour to put back so that it misses no more often than Belady's optimum.
// Solution: a "stay" is one colour's spell on the scaffold (K initial stays, one per request);
//   its bit says whether the colour is requested again before the stay ends. Program 1 replays
//   Belady (evict the colour needed latest) with a Set of (next use, colour), O(N log N).
//   Program 2 may evict any colour whose stay is not needed again: it keeps those on a stack
//   of spares and pops one per miss, O(N).
// original: corpus/submissions/4951033.cpp (two grader functions)
// stdin here: n k, then the n requested colours. stdout: the n + k advice
// bits on one line, then the colour the assistant puts back at each step
// where the requested colour is not on the scaffold.
let n: 1..=100_000 = read()
let k: 1..=n = read()
domain Colour = 0..n
let req: [0..n]Colour = read()

// ---- ComputeAdvice: replay Belady's rule (evict the colour needed latest).
// A "stay" is one colour's unbroken spell on the scaffold: stay i < k is the
// initial colour i, stay k + t starts at request t. Its advice bit says
// whether the colour is requested again before that stay ends.
var next_use: [0..n]Int = n
var upcoming: [Colour]Int = n
for t in (0..n).rev:
    next_use[t] = upcoming[req[t]]
    upcoming[req[t]] = t
var stay: [Colour]Int = -1
var needed: [0..n + k]Bool = false
var on = Set<(Int, Colour)>()
for c in 0..k:
    stay[Colour(c)] = c
    on.insert((upcoming[Colour(c)], Colour(c)))
for t in 0..n:
    let c = req[t]
    if stay[c] != -1:
        needed[stay[c]] = true
        on.remove((t, c))
    else:
        let (_, out) = on.max()!
        on.remove(on.max()!)
        stay[out] = -1
    stay[c] = k + t
    on.insert((next_use[t], c))
print([Int(b) for b in needed])

// ---- Assist: sees only the bits and one request at a time. Colours whose
// current stay is not needed again are safe to put back.
var here: [Colour]Bool = false
var spare: List<Colour> = []
for c in 0..k:
    here[Colour(c)] = true
    if not needed[c]: spare.push(Colour(c))
var put_back: List<Colour> = []
for t in 0..n:
    let c = req[t]
    if not here[c]:
        let out = spare.pop()!
        put_back.push(out)
        here[out] = false
        here[c] = true
    if not needed[k + t]: spare.push(c)
print(put_back)
