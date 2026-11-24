// cf2201b, cf2211d: a ranged list printed whole is stored at its range's width, as in the
// matched C++; print widens each element only as it writes it.
let n: 1..=10_000_000 = read()
var out: List<0..=536_870_911> = []
for i in 0..n: out.push(i * 7919 % 536_870_912)
print(out)
