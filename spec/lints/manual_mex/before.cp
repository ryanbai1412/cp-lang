let n: 1..=100 = read()
let a: [0..n]0..n = read()
var seen: [0..=n]Bool = false
for x in a:
    seen[x] = true
var m = 0
while seen[m]:
    m += 1
print(m)
