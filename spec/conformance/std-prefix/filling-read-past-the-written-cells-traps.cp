// expect: R0401 at 5
// rule: R07.1.6
// Reading a filling table through a cell not yet written traps, even when every earlier cell is written.
domain V = 1..=3
var f = Prefix<V, Int>.filling()
f.set(1, 5)
print(f[V(3)])
