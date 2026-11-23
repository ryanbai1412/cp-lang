let n: 1..=2e5 = read()
let talks: [0..n](s: 0..=1e9, e: 0..=1e9) = read()
var count = 0
var free = 0
for t in talks.sorted(by: t => t.e):
    if t.s >= free:
        count += 1
        free = t.e
print(count)
