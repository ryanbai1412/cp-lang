// Codeforces 2261 G, tuned: port.cp's trie as two flat 32-bit lists (children paired, as orig.cpp's array<int, 2>) with -1 for no child,
// sized once per test, and the suffix-AND groups narrowed in place, as orig.cpp does,
// instead of `?Int` children, a `Node` pushed per bit and a new group list per element.
const L = 25
const FULL = (1 << L) - 1

@!multitest
let n: 1..=4e5 = read()
let a: [0..n]0..=FULL = read()
let size = (L + 1) * (n + 1)
var cnt: List<0..=400_001> = List.filled(size, 0)
var kid: List<-1..10_400_026> = List.filled(2 * size, -1)  // node x's child b at 2 * x + b

fn merge(x: Int, y: Int) -> Int:
    if x == -1: return y
    if y == -1: return x
    cnt[x] += cnt[y]
    for b in 0..2: kid[2 * x + b] = merge(kid[2 * x + b], kid[2 * y + b])
    x

var masks = List<Int>()
var roots = List<Int>()
var (pref, ans) = (0, 0)
for (i, x) in a.items:
    let base = i * (L + 1)
    cnt[base] = 1
    for z in 0..L:
        cnt[base + z + 1] = 1
        kid[2 * (base + z + 1) + ((pref >> z) & 1)] = base + z
    masks.push(FULL)
    roots.push(base + L)
    var len = 0
    for g in 0..masks.len:
        let m = masks[g] & x
        if len > 0 and masks[len - 1] == m:
            roots[len - 1] = merge(roots[len - 1], roots[g])
        else:
            masks[len] = m
            roots[len] = roots[g]
            len += 1
    masks.truncate(len)
    roots.truncate(len)
    pref ^= x
    for g in 0..len:
        let (m, want) = (masks[g], pref ^ masks[g])
        var t = roots[g]
        for b in (0..L).rev:
            t = kid[2 * t + ((want >> b) & 1)]
            if t == -1: break
            if m.bit(b): ans += cnt[t] << b
print(ans)
