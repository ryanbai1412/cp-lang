// expect: ok, output "3 9 6 3 0 2"
// rule: 01.8.4
// `by` steps a domain or a reversed walk too, not only a range: `V by 2` walks every second key, `(0..10).rev by 3` every third value from the top.
domain V = [6]
var s = 0
for v in V by 2:
    s += 1
let r = List((0..10).rev by 3)
var t = 0
for v in V.rev by 4:
    t += 1
print(s, r, t)
