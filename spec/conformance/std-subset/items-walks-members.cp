// expect: ok, output "1\n3"
// `s.items` yields the members of a `Subset` in order, so a `for` walks it (rebuild round 33 F5).
domain B = 0..4
let s = Subset<B>.single(B(3)).with(B(1))
for b in s.items: print(b)
