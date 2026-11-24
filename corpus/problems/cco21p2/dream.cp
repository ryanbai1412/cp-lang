// CCO 2021 P2 Weird Numeral System: base k with an allowed digit set. Write each query x as
// digits from the set (most significant first), or say IMPOSSIBLE.
// The last digit b must satisfy b ≡ x (mod k); then the rest represents (x - b) / k. Search that
// implicit graph from x a layer at a time until 0. A hash map of back pointers over the unbounded
// `Int` states takes over 10 s on k = 2 with all 5001 digits, but a layer below the first lies in
// a window of 2m + 8 numbers, so it is a bitset: one residue's numbers shift together into the
// next layer for each digit of that residue. Near 0 a number joins only the first layer that
// reaches it, so the search ends; walking back from 0 through the layers reads the digits.
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
