// Batch #14: `xs.items`, `xs.find(test)`, a list that fills an array of lists, `Comb<Z.MOD>`.
let n: Int = read()
domain I = 0..n
let a: [I]Int = read()
for (i, x) in a.items: print(i, x)
let b: [0..n]Int = [0..n] i => i * 2
for (i, x) in b.items: print(i + 1, x)
print([x * i.ord for (i, x) in a.items])
let xs = [5, 8, 11]
for (i, x) in xs.items: print(i, x)
let kept = xs.items
print(kept.len, kept[1].0, kept[1].1)
var c: [1..=3]Int = 4
c[2] = 9
for (i, x) in c.items: print(i, x)
for p in a.items: print(p.0, p.1)
fn evens(n: Int) -> List<Int> = [i * 2 for i in 0..n]
print(sum(i * x for (i, x) in evens(4).items))
print(xs.find(x => x > 6), xs.find(x => x > 60), xs.find(8))
print(a.find(x => x % 2 == 0), a.find(x => x < 0))
use std.mod
type Z = Mod<998244353>
let comb = Comb<Z>()
let picked: Z = comb.choose(5, 2)
print(picked)
var ways: [0..n]List<Z> = [Z(1)]
ways[0].push(Z(2))
print(ways)
var g: [I]List<Int> = [7, 8]
g[I.first].push(9)
for i in I: print(g[i])
