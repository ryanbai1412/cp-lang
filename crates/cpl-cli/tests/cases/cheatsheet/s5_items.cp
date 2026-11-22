let n: 1..=10 = read()
let a: [0..n]Int = read()
var out = ""
for (i, x) in a.items: out += "{i}:{x} "
print(out)
