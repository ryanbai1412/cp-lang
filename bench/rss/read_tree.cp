// aac5p3's input (gen.py --max, 6.4 MB), read and summed.
let (n, k): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let hills: [n - 1](u: V, v: V, d: 1..=1e9) = read()
let skills = read<[k]1..=1e9>()
var total = 0
for (_, _, d) in hills: total += d
for s in skills: total += s
print(total)
