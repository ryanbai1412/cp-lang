// Runnable port (proto/cp.py lib-run) of statement-first noimps23p1.cp
// original: corpus/submissions/5556659.cpp
let (n, m, start1): (Int, Int, Int) = read()
var ivs: List<(l: Int, r: Int)> = []
for _ in 0..m:
    let (l, r): (Int, Int) = read()
    ivs.push((l: l - 1, r: r - 1))

// Right endpoints reachable from `start` by riding intervals rightwards.
fn reach(start: Int, iv: List<(l: Int, r: Int)>) -> List<Bool>:
    var starts_at: List<List<Int>> = [[] for _ in 0..n]
    for i in 0..m: starts_at[iv[i].l].push(i)
    // nxt[x]: smallest unswept position >= x (path-compressed skip pointers).
    var nxt: List<Int> = [x for x in 0..n + 1]
    var seen = List.filled(m, false)
    var hit = List.filled(n, false)
    var stack: List<Int> = []
    for i in 0..m:
        if iv[i].l <= start and start < iv[i].r:
            seen[i] = true
            stack.push(i)
    while stack.len > 0:
        let i = stack.pop()
        hit[iv[i].r] = true
        var x = max(start, iv[i].l)
        while true:
            var root = x
            while nxt[root] != root: root = nxt[root]
            while nxt[x] != root:
                let y = nxt[x]
                nxt[x] = root
                x = y
            x = root
            if x > iv[i].r: break
            nxt[x] = x + 1
            for j in starts_at[x]:
                if not seen[j]:
                    seen[j] = true
                    stack.push(j)
    hit[start] = false
    return hit

let start = start1 - 1
let right = reach(start, ivs)
var mirrored: List<(l: Int, r: Int)> = [(l: n - 1 - iv.r, r: n - 1 - iv.l) for iv in ivs]
let left = reach(n - 1 - start, mirrored)
for x in 0..n:
    if right[x] or left[n - 1 - x]: print(x + 1)
