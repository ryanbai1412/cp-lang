domain V = 1..=3
for v in V:
    print(Int(v) + 1)
domain Deg = 0..=2
let d = Deg(1)
let k = 1
print(Deg(d.ord + k))
let big: U64 = 7
print(V(Int(big % U64(3)) + 2))
