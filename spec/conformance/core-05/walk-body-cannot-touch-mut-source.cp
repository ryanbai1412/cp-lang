// expect: E0602
// rule: 05.9.1 05.7.7
fn mark(mut seen: List<Bool>) -> yields Int:
    for i in seen.keys if not seen[i]:
        yield i
var seen = [false, false]
for v in mark(mut seen):
    print(seen[v]) // <- here
