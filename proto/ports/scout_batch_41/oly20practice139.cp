// OLY 2020 Practice 139: choose prices (px, py) for two options given n
// customers' valuations (x, y); customers with x >= px pay px for A, and
// remaining customers with y >= py pay py for B. Maximize total revenue.
// original: corpus/submissions/6415690.cpp
const B = 400

type Line = (m: Int, b: Int)
type Entry = (at: Int, line: Line)

fn eval(l: Line, x: Int) -> Int = l.m * x + l.b

fn ceil_div(a: Int, b: Int) -> Int = (a + b - 1) / b

struct Block:
    ys: List<Int>
    v: List<Int>
    add: Int = 0
    hull: List<Entry> = []
    ptr: Int = 0

    fn rebuild(mut self):
        self.hull.clear()
        self.ptr = 0
        for i in 0..self.ys.len:
            let cur: Line = (m: self.ys[i], b: self.ys[i] * self.v[i])
            while let top = self.hull.last and eval(cur, top.at) > eval(top.line, top.at):
                self.hull.pop()
            let at = if let top = self.hull.last:
                ceil_div(top.line.b - cur.b, cur.m - top.line.m)
            else: 0
            self.hull.push((at:, line: cur))

    fn add_prefix(mut self, k: Int):
        for i in 0..k: self.v[i] += 1
        self.rebuild()

    fn query(mut self) -> Int:
        while self.ptr + 1 < self.hull.len and self.add >= self.hull[self.ptr + 1].at:
            self.ptr += 1
        eval(self.hull[self.ptr].line, self.add)

let n: 1..=200_000 = read()
let pts: [0..n](x: 0..=1_000_000_000, y: 0..=1_000_000_000) = read()

var by_x = Map<Int, List<Int>>()
for p in pts: (by_x[p.x] ??= []).push(p.y)

let ys = compress(p.y for p in pts)
var blocks: List<Block> = []
for ql in (0..ys.D.K).step(B):
    let qr = min(ys.D.K, ql + B)
    var b = Block(ys: [ys[ys.D.from_idx(i)!] for i in ql..qr], v: List.filled(qr - ql, 0))
    b.rebuild()
    blocks.push(b)

fn max_b() -> Int:
    var best = 0
    for i in 0..blocks.len: best max= blocks[i].query()
    best

var rem = n
var ans = 0
for (x, group) in by_x.items:
    ans max= rem * x + max_b()
    for y in group:
        let pos = ys.at(y).to_idx + 1
        for bi in 0..(pos / B): blocks[bi].add += 1
        if pos % B > 0: blocks[pos / B].add_prefix(pos % B)
    rem -= group.len
ans max= max_b()
print(ans)
