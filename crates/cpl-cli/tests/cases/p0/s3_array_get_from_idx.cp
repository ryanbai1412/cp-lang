// Synthesis #3 Leap 3: optional-accepting `a.get(?I)` (1D, 2D, List) and trapping `D.from_idx(i) -> D`.
let n, m: 1..=10 = read()
domain I = 1..=n
domain R = 1..=2
domain C = 1..=m
let a: [I]Int = read()
let g: [R, C]Int = read()

// 1D optional-accepting `.get(?I)` and non-optional `.get(I)`
var acc = 0
for i in I:
    let prev = a.get(i - 1) ?? 0
    let cur = a.get(i) ?? 0
    let nxt = a.get(i + 1) ?? 0
    acc += prev + cur * 10 + nxt

// 2D optional-accepting `.get(?R, C)` / `.get(R, ?C)`
let g1 = g.get(R.first - 1, C.first) ?? -1
let g2 = g.get(R.first + 1, C.first + 1) ?? -1
let g3 = g.get(R.last, C.last + 1) ?? -1

// List.get(Int) and List.get(?Int)
let xs: List<Int> = [100, 200, 300]
let l1 = xs.get(1) ?? -1
let l2 = xs.get(5) ?? -1
let oi: ?Int = 2
let l3 = xs.get(oi) ?? -1
let ni: ?Int = none
let l4 = xs.get(ni) ?? -1

// Trapping `I.from_idx(i) -> I` (no `!`) + fallible `I.try_from_idx(i) -> ?I`
let f0: I = I.from_idx(0)
let flast: I = I.from_idx(n - 1)
let is_oob = I.try_from_idx(n) == none
let neg_idx = I.try_from_idx(-1) == none

print(acc, g1, g2, g3, l1, l2, l3, l4, Int(f0), Int(flast), Int(is_oob), Int(neg_idx))
