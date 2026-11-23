// expect: ok, output "1 3\n2"
// `for b in s` walks a `Subset`'s members in `B`'s order, the same as `s.items`.
domain B = 0..4
let s = Subset<B>.single(B(3)).with(B(1))
var seen: List<B> = []
for b in s: seen.push(b)
print(seen)
print(len([b for b in s]))
