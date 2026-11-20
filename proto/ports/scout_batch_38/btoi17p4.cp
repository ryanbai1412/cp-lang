// Baltic OI '17 P4 (Friends): partition vertices of an undirected graph into
// non-empty groups of size <= p each with <= q edges leaving the group, or
// print "detention". Bounded search tree per vertex + submodular uncrossing.
// original: corpus/submissions/4523876.cpp
let (n, p, q): (1..=2500, 1..=16, 0..=16) = read()
domain V = 0..n
var adj: [V]List<V> = []
var adj_set: [V]HashSet<V> = [V] _ => HashSet<V>()
var bad_input = false
for u in V:
    let deg: 0..=n = read()
    if deg > p + q - 1: bad_input = true
    for _ in 0..deg:
        let v: V = read()
        adj[u].push(v)
        adj_set[u].add(v)

enum State = Undecided | In | Out

fn solve() -> ?List<Set<V>>:
    if bad_input: return none
    for u in V:
        for v in adj[u]:
            if u not in adj_set[v]: return none

    var state: [V]State = Undecided
    var incl: List<V> = []
    var found: ?Set<V> = none

    fn dfs():
        if found != none or incl.len > p: return
        var cut = 0
        for u in incl:
            for v in adj[u]:
                match state[v]:
                    Undecided =>
                        // FRICTION 38.9: no anonymous block for `defer`, so sequential
                        // backtracking branches cannot scope `defer` without a helper fn.
                        state[v] = In
                        incl.push(v)
                        dfs()
                        incl.pop()
                        state[v] = Out
                        dfs()
                        state[v] = Undecided
                        return
                    Out =>
                        cut += 1
                        if cut > q: return
                    In => ()
        var blk = Set<V>()
        for u in incl: blk.add(u)
        found = blk

    var blocks: [V]Set<V> = [V] _ => Set<V>()
    for u in V:
        incl = [u]
        state[u] = In
        found = none
        dfs()
        state[u] = Undecided
        blocks[u] = found?

    fn valid(blk: Set<V>) -> Bool:
        if blk.len > p: return false
        var cut = 0
        for u in blk:
            for v in adj[u]:
                if v not in blk:
                    cut += 1
                    if cut > q: return false
        true

    for i in V:
        for j in i.r..:
            let diff = blocks[i] - blocks[j]
            if valid(diff): blocks[i] = diff
            else: blocks[j] = blocks[j] - blocks[i]

    [blocks[i] for i in V if not blocks[i].is_empty]

if let ans = solve():
    print("home")
    print(ans.len)
    for blk in ans:
        print("{blk.len} " + " ".join("{v}" for v in blk))
else:
    print("detention")
