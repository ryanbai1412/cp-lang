// expect: ok, output "0 2 4 6 6"
// rule: 07.2.10
fn evens(n: Int) -> yields Int = (0..n).filter(x => x % 2 == 0)
var s: List<Int> = []
for x in evens(7): s.push(x)
print(s, evens(5).sum())
