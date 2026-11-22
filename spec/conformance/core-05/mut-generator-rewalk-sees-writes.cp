// expect: ok, output "2\n0"
// rule: 05.9.4 06.3.1 07.6.2
fn mark(mut seen: List<Bool>) -> yields Int:
    for i in seen.keys if not seen[i]:
        seen[i] = true
        yield i
var seen = [false, false]
let g = mark(mut seen)
print(g.count(x => true))
print(g.count(x => true))
