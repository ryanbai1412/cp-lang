// ioi20p5's shape: a payload enum matched per element of a list, n masks
// over a 12-slot query, under `cpl run`.
enum Slot: Known(0..=1) | Unknown(Int)

fn species(s: Slot, mask: Int) -> Int:
    match s:
        Known(t) => t
        Unknown(i) => (mask >> i) & 1

let n: Int = read()
let q = [Unknown(i) for i in 0..6] + [Known(0) for _ in 0..3] + [Known(1) for _ in 0..3]
var acc = 0
for mask in 0..n:
    acc += (1..q.len).count(i => species(q[i - 1], mask) != species(q[i], mask))
print(acc)
