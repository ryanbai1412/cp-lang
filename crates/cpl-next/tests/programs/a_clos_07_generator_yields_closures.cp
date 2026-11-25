// expect: ok, output "1 2 3"
// rule: R05.10.1, R05.10.6
// A generator local var captured by a closure that is yielded and called by the body (o-series neighbour).
fn gen() -> yields () -> Int:
    var c = 0
    for i in 0..3:
        yield () => { c += 1; c }
var out: List<Int> = List.new()
for f in gen(): out.push(f())
print(out)
