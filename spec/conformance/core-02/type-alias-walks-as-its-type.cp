// expect: ok, output "8"
// rule: 02.6.2
// A `type` alias is the type it names: walking an alias of `Subset<R>` walks the subsets.
domain R = 1..=3
type G = Subset<R>
var n = 0
for s in G:
    n += 1
print(n)
