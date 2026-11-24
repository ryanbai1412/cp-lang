// expect: R0001 (checked)
// Entries past 2^60 leave the flat max-plus pass to the loop, whose add traps on an overflow as written.
domain V = 0..2
let big = 1 << 62
let d: [V, V]?Int = [V, V] (i, j) => if i == j: Some(big) else: None
let m = Mat(d, over: Semiring(max, +))  // <- here
print((m * m).get(0, 0) ?? -1)
