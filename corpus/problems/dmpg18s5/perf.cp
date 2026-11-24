// DMPG 2018 S5, tuned: port.cp with the partial-block scan testing divisibility in `U64`, unsigned, instead of
// a signed `Int` remainder, whose sign fix-up costs a division per value.
const MAX = 200_000
const SIDE = 500

let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain Pos = 1..=n
domain Val = 1..=MAX
domain Block = 0..ceil_div(n, SIDE)
var a: [Pos]Val = read()

var divisors: [Val]List<Val> = []
for d in Val:
    for v in d.label..=MAX by d.label: divisors[Val(v)].push(d)
fn block(p: Pos) -> Block = Block((p.label - 1) / SIDE)
fn first(b: Block) -> Pos = Pos(b.label * SIDE + 1)
fn last(b: Block) -> Pos = Pos(min(n, (b.label + 1) * SIDE))

var count: [Val, Block]0..=SIDE = 0  // how many of the block's values each divisor divides
fn shift(p: Pos, by: Int):
    for d in divisors[a[p]]: count[d, block(p)] += by
for p in Pos: shift(p, 1)

fn scan(lo: Pos, hi: Pos, x: Val) -> Int = (lo..=hi).count(p => U64(a[p].label) % U64(x.label) == U64(0))

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let l: Pos = read()
        let r: Pos = read()
        let x: Val = read()
        let (bl, br) = (block(l), block(r))
        if bl == br: print(scan(l, r, x))
        else:
            let inside = sum(count[x, Block(b)] for b in bl.label + 1..br.label)
            print(scan(l, last(bl), x) + inside + scan(first(br), r, x))
    else:
        let p: Pos = read()
        let v: Val = read()
        shift(p, -1)
        a[p] = v
        shift(p, 1)
