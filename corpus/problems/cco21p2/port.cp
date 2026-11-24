// Source: CCO 2021 Day 1 P2 (CCO '21 P2), Weird Numeral System, https://dmoj.ca/problem/cco21p2
// limits: 1500 ms, 1024 MB
// Problem: a base K (2 <= K <= 1e6) and D <= 5001 distinct digits a_i with |a_i| <= M <= 2500.
//   For each of Q <= 5 numbers n (|n| <= 1e18) print, on one line and space-separated, digits
//   most significant first whose base-K value is n (any such; 0 needs at least one digit), or
//   IMPOSSIBLE.
// Solution: the last digit b of x must be congruent to x mod K, and the other digits then
//   spell (x - b) / K. Search that graph breadth first from x, a layer at a time, until 0 is
//   reached by at least one digit. A layer below the first lies in a window of 2M + 8 numbers,
//   so it is a bitset: its numbers y = y0 + K u of one residue, as the bits u, shift by
//   (y0 - b) / K into the next layer for each digit b of that residue, D word-parallel shifts
//   per layer. The numbers shrink to |y| <= M / (K - 1) + 1 within about 60 layers; inside that
//   range a number joins only the first layer that reaches it, so the search ends. Walking back
//   from 0 through the kept layers reads the digits, most significant first.
// original: corpus/submissions/4543410.cpp
let (k, q, d, m): (2..=1e6, 1..=5, 1..=5001, 1..=2500) = read()
domain Residue = 0..k
let digits: [d](-m..=m) = read()
var ending: [Residue]List<Int> = []
for b in digits: ending[Residue(b % k)].push(b)
domain Window = 0..2 * m + 8
let near = m / (k - 1) + 1
domain Near = -near..=near

// The numbers lo + i for i in bits.
struct Layer:
    lo: Int
    bits: Bitset<Window>

    fn has(self, y: Int) -> Bool = y - lo in 0..Window.count and bits.contains(Window(y - lo))

fn step(cur: Layer, mut seen: Bitset<Near>) -> Layer:
    let lo = (cur.lo - m) / k
    var bits = Bitset<Window>()
    var by_residue = [(y % k, y) for y in cur.bits.each().map(i => cur.lo + i.ord)]
    by_residue.sort()
    for group in by_residue.runs(by: p => p.0):
        let (r, y0) = group.first!
        var us = Bitset<Window>()
        for (_, y) in group: us.add(Window((y - y0) / k))
        for b in ending[Residue(r)]: bits.union_shifted(us, (y0 - b) / k - lo)
    for i in bits.each().collect():
        let y = lo + i.ord
        if y in Near and not seen.add(Near(y)): bits.remove(i)
    Layer(lo, bits)

fn spell(x: Int) -> ?List<Int>:
    var layers = [Layer(x, Bitset<Window>.single(Window(0)))]
    var seen = Bitset<Near>()
    while layers.len == 1 or not layers.last!.has(0):
        if layers.last!.bits.first() == None: return None
        layers.push(step(layers.last!, mut seen))
    var out: List<Int> = []
    var y = 0
    for layer in layers[..layers.len - 1].rev:
        let b = digits.find(b => layer.has(y * k + b))!
        out.push(b)
        y = y * k + b
    out

for _ in 0..q:
    let x: -1e18..=1e18 = read()
    print(spell(x), none: "IMPOSSIBLE")
