// CF 1922D Berserk Monsters: only neighbours of the previous round's dead can die next, so
// keep a linked list of the living and re-check just those candidates each round.
@!multitest
let n: 1..=3e5 = read()
domain V = 1..=n
let a = read<[V]1..=1e9>()
let d = read<[V]1..=1e9>()
var prv: [V]?V = [V] v => v.prev
var nxt: [V]?V = [V] v => v.next
var alive: [V]Bool = true
var cand = [v for v in V]
var deaths = List<Int>()
repeat n:
    var dead = List<V>()
    for v in cand:
        if alive[v]:
            let hit = (prv[v].map(u => a[u]) ?? 0) + (nxt[v].map(u => a[u]) ?? 0)
            if hit > d[v]:
                alive[v] = false
                dead.push(v)
    for v in dead:
        if let p = prv[v]: nxt[p] = nxt[v]
        if let q = nxt[v]: prv[q] = prv[v]
    cand = []
    for v in dead:
        if let p = prv[v], alive[p]: cand.push(p)
        if let q = nxt[v], alive[q]: cand.push(q)
    deaths.push(dead.len)
print(deaths)
