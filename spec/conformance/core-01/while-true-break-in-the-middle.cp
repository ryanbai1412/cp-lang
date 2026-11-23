// expect: ok, output "1 2 4 8"
// rule: 01.7.2
// There is no `loop`; `while true:` with a `break` exits in the middle (rebuild round 15 H-WHILE-TRUE).
var x = 1
var out: List<Int> = []
while true:
    out.push(x)
    if x >= 8: break
    x *= 2
print(out)
