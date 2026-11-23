let n: 1..=100 = read()
let a: [0..n]0..n = read()
var seen: [0..=n]Bool = false
for x in a:
    seen[x] = true
print((0..=n).find(m => not seen[m])!)
