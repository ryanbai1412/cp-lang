// Synthesis #2 rule 3 + Synthesis #3 L3.3: trapping V.from_idx(i) -> V and fallible V.try_from_idx(i) -> ?V.
let n: 1..=10 = read()
domain V = 1..=n
let v = V(2)                       // from the written label, checked
print(Int(v), v.to_idx, v)         // the label back; the 0-based position; print shows the label
let first = V.from_idx(0)          // -> V, trapping inverse of to_idx
let past = V.try_from_idx(n)       // -> ?V, fallible inverse
let neg = V.try_from_idx(-1)
print(Int(first), Int(past == None), Int(neg == None))
var tot = 0
for u in V: tot += Int(u) * 10 + u.to_idx
print(tot, Int('a'), Int(true))
if let w = V.try_from_idx(3): print(w, Int(w) + 1)
