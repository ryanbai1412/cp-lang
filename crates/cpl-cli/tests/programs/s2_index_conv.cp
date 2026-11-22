// Synthesis #2 rule 3: the four index conversions that stay.
let n: 1..=10 = read()
domain V = 1..=n
let v = V(2)                       // from the written label, checked
print(Int(v), v.to_idx, v.label)         // the label back; the 0-based position; print shows the label
let first = V.at(1)
let past = V.at(n + 1)
let neg = V.at(0)
print(Int(first!), Int(past == none), Int(neg == none))
var tot = 0
for u in V: tot += Int(u) * 10 + u.to_idx
print(tot, Int('a'), Int(true))
if let w = V.at(4): print(w.label, Int(w) + 1)
