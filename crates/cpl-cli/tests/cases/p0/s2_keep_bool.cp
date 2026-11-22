// Synthesis #2 rule 1: `x min= y` / `x max= y` is a Bool, true when the place changed.
let n: 1..=10 = read()
domain V = 0..n
var dist: [V]?Int = none
var ways: [V]Int = 0
dist[V(0)] = 5
if dist[V(0)] min= 3: ways[V(0)] = 1          // 5 -> 3: changed
if dist[V(0)] min= 7: ways[V(0)] = 99         // stays 3: not changed
if dist[V(1)] min= 4: ways[V(1)] = 2          // none takes the value: changed
if dist[V(1)] min= none: ways[V(1)] = 98      // none on the right is skipped
print(dist[V(0)]!, ways[V(0)], dist[V(1)]!, ways[V(1)])

// a record compares field by field, so the id rides along with the key
var best = (gap: 9, id: 0)
let a = best min= (gap: 4, id: 7)
let b = best min= (gap: 4, id: 8)
print(best.gap, best.id, Int(a), Int(b))

var hi = 0
var grew = 0
for x in [3, 1, 4, 1, 5]:
    if hi max= x: grew += 1
print(hi, grew)

// the update happens where it is written: not at all on the unused side of `and`
var lo = 10
let skipped = false and lo min= 1
let taken = true and lo min= 6
print(lo, Int(skipped), Int(taken))

// a key that is not in the map yet takes the value
var m = Map<Int, Int>()
let first = m[2] max= 5
let same = m[2] max= 5
let more = m[2] max= 6
print(m[2], Int(first), Int(same), Int(more))

// as a statement the Bool is dropped, as before
lo min= 2
print(lo)
