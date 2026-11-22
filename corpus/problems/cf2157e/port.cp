// Codeforces 2157 E — Value-Spreading Stabilization Time
//
// Each round increments every value occurring strictly more than `k` times by
// 1 (keeping one copy behind and moving the excess copies up by 1), until all
// frequencies are `<= k`.
//
// Sweeping `v` from `2 * n` down to `1`, maintain `stk` as the stack of empty
// slots in `v..=2 * n` (nearest slot on top), with `next_free` supplying fresh
// slots `2 * n + 1, 2 * n + 2, ...` whenever `stk` is empty. Each copy of `v`
// claims the nearest free slot `slot >= v`; the `k`-th remaining copy of `v`
// (when `rem == k`) is the last copy forced to advance, so it needs `slot - v`
// rounds to reach `slot`. The maximum over all `v` is the stabilization time.

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
domain Val = 1..=2 * n
let a: [0..n]Val = read()
var freq: [Val]Int = 0
for x in a: freq[x] += 1

var stk: List<Int> = []
var next_free = 2 * n
var ans = 0
for v in Val.rev:
    stk.push(v.ord)
    var rem = freq[v]
    while rem > 0:
        let slot = if let s = stk.pop():
            s
        else:
            let s = next_free
            next_free += 1
            s
        if rem == k: ans max= slot - v.ord
        rem -= 1
print(ans)
