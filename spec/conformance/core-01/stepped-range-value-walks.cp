// expect: ok, output "2 5 8\n10 7 4 1\n3 4"
// rule: 01.8.4
// A stepped range kept in a variable walks like the loop header's, either way, and counts in O(1).
let r = 2..9 by 3
var out: List<Int> = []
for x in r: out.push(x)
print(out)
let n = 10
let down = (1..=n by 3).rev
print(down)
print(r.len, down.len)
