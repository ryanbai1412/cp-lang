// expect: ok, output "4"
// rule: 01.6.2
let xs: List<?Int> = [1, None, 3, None, 9, 2]
var s = 0
for x in xs:
    let v = x else: continue
    if v > 5: break
    s += v
print(s)
