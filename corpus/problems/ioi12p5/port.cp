// Source: IOI 2012 Day 2 (Last Supper), https://dmoj.ca/problem/ioi12p5
// limits: 2000 ms, 256 MB
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
// Bounds: n: 1..=100_000; k: 1..=n; req: 0..=n - 1
let n: 1..=100_000 = read()
let k: 1..=n = read()
domain Colour = 0..n
domain Step = 0..n
domain Stay = 0..n + k
let req: [Step]Colour = read()

// stay i < k is the initial colour i; stay k + t starts at request t
fn opened(t: Step) -> Stay = Stay.nth(k + t.ord)
fn initial(c: Colour) -> ?Stay = if c.ord < k: Some(Stay.nth(c.ord)) else: None

// ---- ComputeAdvice: replay Belady's rule (evict the colour needed latest).
// Times run 0..=n, where n is "never again". A stay's advice bit says
// whether its colour is requested again before the stay ends.
var next_use: [Step]Int = n
var upcoming: [Colour]Int = n
for t in Step.rev:
    next_use[t] = upcoming[req[t]]
    upcoming[req[t]] = t.ord
var stay: [Colour]?Stay = [Colour] c => initial(c)
var needed: [Stay]Bool = false
var on = Set<(Int, Colour)>()
for c in Colour:
    if stay[c] != None: on.insert((upcoming[c], c))
for t in Step:
    let c = req[t]
    if let s = stay[c]:
        needed[s] = true
        on.remove((t.ord, c))
    else:
        let (_, out) = on.pop_last()!
        stay[out] = None
    stay[c] = Some(opened(t))
    on.insert((next_use[t], c))
print([Int(b) for b in needed])

// ---- Assist: sees only the bits and one request at a time. Colours whose
// current stay is not needed again are safe to put back.
var here: [Colour]Bool = [Colour] c => initial(c) != None
var spare: List<Colour> = [c for c in Colour if let s = initial(c), not needed[s]]
var put_back: List<Colour> = []
for t in Step:
    let c = req[t]
    if not here[c]:
        let out = spare.pop()!
        put_back.push(out)
        here[out] = false
        here[c] = true
    if not needed[opened(t)]: spare.push(c)
print(put_back)
