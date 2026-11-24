// expect: ok, output "0\n1\n2\n0 1 2 3"
// rule: 04.4.30
// `for k in ..3` walks the `Int`s 0, 1, 2; `..=3` takes 3 too.
for k in ..3: print(k)
var seen: List<Int> = []
for k in ..=3: seen.push(k)
print(seen)
