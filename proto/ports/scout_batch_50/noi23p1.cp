// NOI 2023 P1, count the lattice cells covered by horizontal, vertical and diagonal strokes
// original: corpus/submissions/5723056.cpp
type Stroke = (line: Int, lo: Int, hi: Int)
let (tc, rows, cols, n): (Int, 1..=1e9, 1..=1e9, 1..=2e5) = read()
var drawn: [1..=3]List<Stroke> = []     // 1: (y, x..x), 2: (x, y..y), 3: (y - x, x..x)
for _ in 0..n:
    let (kind, x1, y1, x2, y2): (1..=3, Int, Int, Int, Int) = read()
    drawn[kind].push(match kind:
        1 => (y1, min(x1, x2), max(x1, x2))
        2 => (x1, min(y1, y2), max(y1, y2))
        3 => (y1 - x1, min(x1, x2), max(x1, x2)))

// strokes on one line that overlap or touch become one stroke
fn merged(strokes: List<Stroke>) -> List<Stroke>:
    var out: List<Stroke> = []
    for s in sorted(strokes):
        if let top = out.last and top.line == s.line and s.lo <= top.hi + 1:
            out[out.len - 1].hi max= s.hi
        else: out.push(s)
    out
let (hori, vert, diag) = (merged(drawn[1]), merged(drawn[2]), merged(drawn[3]))
var cells = sum(s.hi - s.lo + 1 for s in hori + vert + diag)

// a cell on a horizontal and a vertical stroke was counted twice: sweep y
// upward keeping the x of every vertical stroke alive at that height
enum Event = Leave(x: Int) | Enter(x: Int) | Ask(lo: Int, hi: Int)
var events: List<(y: Int, e: Event)> = [(s.lo, Enter(s.line)) for s in vert] +
    [(s.hi + 1, Leave(s.line)) for s in vert] + [(s.line, Ask(s.lo, s.hi)) for s in hori]
var alive = SortedBag<Int>()
for (y, e) in sorted(events):
    match e:
        Leave(x) => alive.remove(x)
        Enter(x) => alive.add(x)
        Ask(lo, hi) => cells -= alive.rank(hi + 1) - alive.rank(lo)

// a diagonal stroke meets each other stroke in at most one cell
for d in diag:
    let on_hori = [s.line - d.line for s in hori if s.line - d.line in s.lo..=s.hi]
    let on_vert = [s.line for s in vert if s.line + d.line in s.lo..=s.hi]
    cells -= Set<Int>(x for x in on_hori + on_vert if x in d.lo..=d.hi).len
print(cells)
