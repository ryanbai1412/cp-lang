// CF1949F Dating. Take users by decreasing set size; with no good match so
// far the sets form a laminar family and last[a] is the smallest set holding a.
// A user is consistent iff every activity's last is the same set x; otherwise
// (u, x) is a good match for x the most recently taken of those sets.
let n: 2..=2e5 = read()
let m: 1..=1e6 = read()
domain U = 1..=n
domain A = 1..=m
let acts = read<[U]List<A>>()
let ord = acts.keys.sorted(by: u => desc(acts[u].len))
var pos: [U]Int = 0
for (i, u) in ord.items: pos[u] = i
var last: [A]?U = None
for u in ord:
    var x: ?U = None
    for a in acts[u]:
        if let y = last[a]:
            let newer = if let xv = x: pos[y] > pos[xv] else: true
            if newer: x = y
    if let w = x:
        if acts[u].any(a => last[a] != x):
            print("YES")
            print(u, w)
            return
    for a in acts[u]: last[a] = u
print("NO")
