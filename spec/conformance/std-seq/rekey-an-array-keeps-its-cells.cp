// expect: ok, output "1 2"
// rule: std.seq.rekey
// `.rekey` on a `[D]T` relabels its cells on both engines.
domain D = 0..3
let a: [D]Int = [D] i => i.ord
let b = a.rekey(10..13)
print(b[11], b[12])
