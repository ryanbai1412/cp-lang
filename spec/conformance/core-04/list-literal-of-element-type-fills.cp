// expect: ok, output "1 0"
// rule: 04.8.10
// `[]` is a `List<Int>`, the element type, so it fills every cell (round 25, H10).
let n = 2
var out: [n + 1]List<Int> = []
out[1].push(4)
print(out[1].len, out[0].len)
