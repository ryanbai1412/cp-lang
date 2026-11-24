// expect: ok, output "0"
// rule: 05.2.1 05.3.5
// An Int read through two index levels holds no lend, so its own place can be written while it lives.
var nxt = [0..3] _ => [0..3] i => i
var cur = 0
let nx = nxt[0][cur]
nxt[0][cur] = 2
cur = nx
print(cur)
