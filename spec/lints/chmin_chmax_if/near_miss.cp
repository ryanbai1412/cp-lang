var best = 0
var low = 9
var total = 0
for v in [3, 1, 4]:
    total += v
    best = max(total, v)
    if v < low: low = total
print(best, low, total)
// On lists `min=` is elementwise while `<` is lexicographic: not this lint.
let cyc = [3, 1, 2]
var least = [3, 1, 2]
for s in 1..3:
    let rot = [cyc[(s + i) % 3] for i in 0..3]
    if rot < least: least = rot
print(least)
// `min=` read as a value is already the form this lint asks for.
for v in [5, 2, 7, 1]:
    if low min= v: print(v)
