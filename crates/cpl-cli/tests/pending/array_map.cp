// `.map` on fixed arrays `[D]T` and `[D, E]T` preserves index domains (`[D]U`, `[D, E]U`);
// `List`, `Str`, and slices continue to return `List<U>`.

enum Side:
    X
    Y

struct Run(at: Int, end: Int, d: Int):
    fn drop(self, k: Int) -> Run = Run(at + d * k, end, d)

struct Block(seen: [Side]Run, real: [Side]Run):
    fn each(self, f: Run -> Run) -> Block = Block(seen.map(f), real.map(f))

// 1. Enum-indexed array mapped via function value (agc077a dream line 52).
let r0 = Run(1, 5, 1)
let r1 = Run(5, 1, -1)
let b0 = Block(
    seen: [Side] s => if s == X: r0 else: r1,
    real: [Side] s => if s == X: r0 else: r1,
)
let b1 = b0.each(v => v.drop(2))
print(b1.seen[X].at, b1.seen[X].end, b1.seen[Y].at, b1.seen[Y].end)

// 2. Domain-indexed 1D array `[D]Int` -> `[D]Int` and `[D]Bool`.
domain D = 0..4
fn sum_d(a: [D]Int) -> Int:
    var s = 0
    for i in D: s += a[i]
    s

let arr: [D]Int = [D] i => i.ord + 1
let doubled: [D]Int = arr.map(x => x * 2)
let even: [D]Bool = arr.map(x => x % 2 == 0)
print(sum_d(doubled))
print(even)

// 3. 1-based range-indexed array `[1..=4]Int` -> `[1..=4]Int`.
let one_based: [1..=4]Int = [1..=4] i => i * 10
let shifted: [1..=4]Int = one_based.map(x => x + 1)
print(shifted[1], shifted[4])

// 4. Empty domain `[Empty]Int` -> `[Empty]Int`.
domain Empty = 0..0
let emp: [Empty]Int = 0
let emp2: [Empty]Int = emp.map(x => x + 1)
print(emp2.len)

// 5. Multi-dimensional `[R, C]Int` -> `[R, C]Int` and `[R, C]Bool`.
domain R = 0..2
domain C = 0..3
fn sum_rc(g: [R, C]Int) -> Int:
    var s = 0
    for r in R:
        for c in C: s += g[r, c]
    s

fn inc(x: Int) -> Int = x + 100
let grid: [R, C]Int = [R, C] (r, c) => r.ord * 10 + c.ord
let grid_lam: [R, C]Int = grid.map(x => x * 2)
let grid_fn: [R, C]Int = grid.map(inc)
let grid_bool: [R, C]Bool = grid.map(x => x >= 10)
print(sum_rc(grid_lam), sum_rc(grid_fn))
print(yes(grid_bool[R(0), C(2)]), yes(grid_bool[R(1), C(0)]))

// 6. Nested `[Side][C]Int` -> `[Side][C]Int`.
let nested: [Side][C]Int = [Side] s => [C] c => if s == X: c.ord else: c.ord + 10
let nested2: [Side][C]Int = nested.map(row => row.map(x => x * 3))
print(nested2[X])
print(nested2[Y])

// 7. List, Str, and slices stay `List<U>`.
let xs: List<Int> = [1, 2, 3, 4]
let ys: List<Int> = xs.map(x => x + 10)
let zs: List<Int> = xs[1..3].map(x => x * 5)
let cs: List<Int> = "ABBA".map(c => Int(c == 'B'))
print(ys)
print(zs)
print(cs)
