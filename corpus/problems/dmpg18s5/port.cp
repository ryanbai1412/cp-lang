// Source: DMPG 2018 S5 Mimi and Division, https://dmoj.ca/problem/dmpg18s5
// Problem: N, Q <= 2e5 and an array A; queries "1 l r x" print how many of A[l..=r] are divisible
//   by x, "2 u v" sets A[u] = v; values and x are at most 2e5.
// Solution: sqrt decomposition with blocks of 500: each block keeps, for every d, how many of
//   its values d divides, updated through the divisors of the old and new value (std
//   `Sieve.divisors`). A query scans its two partial blocks and adds the counts at x of the
//   whole ones between. O((N + Q) (sqrt N + D)), D the most divisors, with N / 500 * V counters.
// original: corpus/submissions/3476383.cpp

const MAX = 200_000
const SIDE = 500

let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain Pos = 1..=n
domain Val = 1..=MAX
domain Block = 0..ceil_div(n, SIDE)
var a: [Pos]Val = read()

let sieve = Sieve.new(MAX)
fn block(p: Pos) -> Block = Block((p.label - 1) / SIDE)
fn first(b: Block) -> Pos = Pos(b.label * SIDE + 1)
fn last(b: Block) -> Pos = Pos(min(n, (b.label + 1) * SIDE))

var count: [Block, Val]0..=SIDE = 0  // how many of the block's values each divisor divides
fn shift(p: Pos, by: Int):
    for d in sieve.divisors(a[p].label): count[block(p), Val(d)] += by
for p in Pos: shift(p, 1)

fn scan(lo: Pos, hi: Pos, x: Val) -> Int = (lo..=hi).count(p => a[p].label % x.label == 0)

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let l: Pos = read()
        let r: Pos = read()
        let x: Val = read()
        let (bl, br) = (block(l), block(r))
        if bl == br: print(scan(l, r, x))
        else:
            let inside = sum(count[Block(b), x] for b in bl.label + 1..br.label)
            print(scan(l, last(bl), x) + inside + scan(first(br), r, x))
    else:
        let p: Pos = read()
        let v: Val = read()
        shift(p, -1)
        a[p] = v
        shift(p, 1)
