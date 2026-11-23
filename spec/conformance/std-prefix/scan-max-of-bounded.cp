// expect: ok, output "3 3 5"
// rule: 07.9.2 07.1.6
// `max` has an identity on a bounded element type, so `a.scan(max)` needs no `from:`; at a position it is the running max (rebuild 8, H-SCAN-MAX-IDENTITY).
domain I = 0..3
let a: [I]1..=9 = [I] i => [3, 1, 5][Int(i)]
let m = a.scan(max)
print(m[I(0)], m[I(1)], m[I(2)])
