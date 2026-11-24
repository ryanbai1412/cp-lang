// Source: EGOI 2022 Day 2 (Toy Design), https://dmoj.ca/problem/egoi22p7
// limits: 1000 ms, 256 MB
// Problem: interactive. n <= 200 pins with hidden wires (design 0). Connected(a, i, j) says
//   whether pins i and j are joined in design a; if not, it creates a new design: a plus wire
//   (i, j). Using at most max_ops = 1350 calls, describe any design whose connectivity equals
//   design 0's.
// Solution: divide and conquer over pin ranges; each range gets a design in which the whole
//   range is joined. To join two halves, start from the right half's design and take one pin
//   per known component of the left half: if Connected says no, keep the new design; if yes,
//   the pin is joined in design 0 to a right component not yet matched, found by descending
//   the right half's ranges with one query per level (skipping a half whose components are
//   all matched). A Dsu records the joins; O(n log n) queries.
// Stdin protocol (the grader embedded, as the accepted C++ builds it with LOCAL): n, m, then
//   the m hidden wires. Prints the query count, then the described design's wires.
// Bounds: n: 1..=200; wires.i, wires.j: 1..=n
// original: corpus/submissions/5203797.cpp

let n: 1..=200 = read()
let m: Int = read()
domain Pin = 1..=n
let wires: [m](i: Pin, j: Pin) = read()

// The grader: design a is designs[a], design 0 holds the hidden wires.
var designs: List<Dsu<Pin>> = [Dsu<Pin>()]
for w in wires: designs[0].unite(w.i, w.j)
var queries = 0

fn connected(a: Int, i: Pin, j: Pin) -> Int:
    queries += 1
    if designs[a].same(i, j): return a
    var wired = designs[a]
    wired.unite(i, j)
    designs.push(wired)
    designs.len - 1

// The solver.
type Span = (lo: Int, hi: Int)  // the pins lo..=hi
fn halves(s: Span) -> (Span, Span):
    let mid = s.lo + (s.hi - s.lo) / 2
    ((s.lo, mid), (mid + 1, s.hi))

var joins = Dsu<Pin>()
var design = Map<Span, Int>()  // a design joining the whole span
var rep = Map<Span, Pin>()

fn among(pins: List<Pin>, p: Pin) -> Bool = pins.any(a => joins.same(a, p))

// The pin of s that x is wired to in design 0, outside the components of `matched`.
fn partner(s: Span, x: Pin, matched: List<Pin>) -> Pin:
    if s.lo == s.hi: return Pin(s.lo)
    let (left, right) = halves(s)
    let open = (h: Span) => (h.lo..=h.hi).any(i => not among(matched, Pin(i)))
    if not open(left): return partner(right, x, matched)
    if not open(right): return partner(left, x, matched)
    let d = design[left]
    if connected(d, x, rep[left]) == d: partner(left, x, matched) else: partner(right, x, matched)

fn build(s: Span):
    if s.lo == s.hi:
        design[s] = 0
        rep[s] = Pin(s.lo)
        return
    let (left, right) = halves(s)
    build(left)
    build(right)
    var d = design[right]
    var seen: List<Pin> = []
    var matched: List<Pin> = []
    for i in left.lo..=left.hi:
        let p = Pin(i)
        if among(seen, p): continue
        seen.push(p)
        let wired = connected(d, p, rep[right])
        if wired != d:
            d = wired
        else:
            let q = partner(right, p, matched)
            matched.push(q)
            joins.unite(p, q)
    design[s] = d
    rep[s] = rep[left]

build((Pin.first!.label, Pin.last!.label))
let described = [(joins.find(p), p) for p in Pin if joins.find(p) != p]
print(queries)
print(described.len)
for (a, b) in described: print(a, b)

@check
fn check():  // any design with design 0's connectivity, in at most 1350 queries
    let used: Int = read_output()
    assert used <= 1350
    let k: Int = read_output()
    var hidden = Dsu<Pin>()
    for w in wires: hidden.unite(w.i, w.j)
    var built = Dsu<Pin>()
    for _ in 0..k:
        let a: Pin = read_output()
        let b: Pin = read_output()
        built.unite(a, b)
    for p in Pin:
        for q in Pin: assert built.same(p, q) == hidden.same(p, q)
