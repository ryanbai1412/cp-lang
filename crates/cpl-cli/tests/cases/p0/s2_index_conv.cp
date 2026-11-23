// Synthesis #2 rule 3 + Synthesis #3 L3.3: trapping V.nth(i) -> V and fallible V.try_at(i) -> ?V.
let n: 1..=10 = read()
domain V = 1..=n
let v = V(2)                       // from the written label, checked
print(Int(v), v.ord, v)         // the label back; the 0-based position; print shows the label
let first = V.nth(0)             // -> V, trapping inverse of ord
let past = V.try_at(n)       // -> ?V, fallible inverse
let neg = V.try_at(-1)
print(Int(first), Int(past == None), Int(neg == None))
var tot = 0
for u in V: tot += Int(u) * 10 + u.ord
print(tot, Int('a'), Int(true))
if let w = V.try_at(3): print(w, Int(w) + 1)
