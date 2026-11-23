// expect: ok, output "0 1 2 3\n2 1"
// `for s in Subset<C>` walks the subsets; `for c in s` walks one subset's members.
domain C = 0..2
var ways: [Subset<C>]Int = 0
for s in Subset<C>:
    ways[s] = s.len
print(List(s.ord for s in Subset<C>))
let full = Subset<C>.full
print(ways[full], List(c for c in Subset<C>.single(C(1))).len)
