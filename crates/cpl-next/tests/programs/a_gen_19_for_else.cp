// expect: ok, output "10"
// rule: R06 loop else
// `while` with `else`, `for` with `else` (R06): else runs when the loop was not left by break.
var s = 0
for i in 0..4:
    s += i
else:
    s += 4
print(s)
