// Source: DMOJ OLY 2020 Practice 139 (Bob's Function), https://dmoj.ca/problem/oly20practice139
// Problem: f(x, y) = a if a <= x, else b if b <= y, else 0. Given n <= 150000 pairs with
//   0 <= y_i <= x_i <= 1e9, choose constants a, b to maximise sum f(x_i, y_i); print the maximum.
// Solution: some x_i is an optimal a (or a is above every x); then the sum is
//   a * #(x >= a) + b * #(x < a, y >= b), and b is best at one of the y values. Sweep a over
//   the sorted x; pairs with x < a join run by run, each adding 1 to joined[v] for every
//   y value v <= its y, and we need max v * joined[v]. Over compressed y, sqrt blocks of
//   B = 400 keep the lines v * (joined[v] + t) as an upper hull (slopes v rise) queried at
//   the block's pending add t, which only grows until the block is rebuilt: O(n sqrt n).
//   Line i of the hull wins from t = from[i], the ceil of its crossing with line i - 1;
//   std's MonotoneHull cross-multiplies and overflows on these sizes (friction).
// original: corpus/submissions/6415690.cpp

const B = 400

let n: 1..=150_000 = read()
let pts = read<[n](0..=1_000_000_000, 0..=1_000_000_000)>().sorted()
let (domain Y, ys) = compress([p.1 for p in pts])
domain Blk = 0..(Y.count + B - 1) / B

struct Upper:
    lines: List<Line> = []
    from: List<Int> = []
    at: Int = 0

    fn clear(mut self):
        lines.clear()
        from.clear()
        at = 0

    fn add(mut self, l: Line):
        while not lines.is_empty and l.at(from.last!) >= lines.last!.at(from.last!):
            let _ = lines.pop()
            let _ = from.pop()
        if let Some(top) = lines.last: from.push((top.m - l.m).ceil_div(l.k - top.k))
        else: from.push(0)
        lines.push(l)

    fn at_next(mut self, t: Int) -> Int:
        while at + 1 < lines.len and t >= from[at + 1]: at += 1
        lines[at].at(t)

var joined: [Y]Int = 0
var pending: [Blk]Int = 0
var hull: [Blk]Upper = Upper()

fn cells(b: Blk) -> Range<Int> = b.ord * B..min((b.ord + 1) * B, Y.count)

fn rebuild(b: Blk):
    hull[b].clear()
    for o in cells(b):
        let v = Y.nth(o)
        hull[b].add(Line(ys[v], ys[v] * joined[v]))

fn gain() -> Int = max(hull[b].at_next(pending[b]) for b in Blk)!

fn join(y: Int):
    let last = ys.index_of(y)
    let b = Blk.nth(last.ord / B)
    for full in Blk.nth(0)..b: pending[full] += 1
    for o in b.ord * B..=last.ord: joined[Y.nth(o)] += 1
    rebuild(b)

for b in Blk: rebuild(b)
var best = 0
var left = n
for run in pts.runs(by: p => p.0):
    best max= left * run.first!.0 + gain()
    for p in run: join(p.1)
    left -= run.len
print(max(best, gain()))
