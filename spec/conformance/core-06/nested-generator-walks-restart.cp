// expect: ok, output "51"
// rule: 05.9.1 13.3.3
// Each walk of a generator reruns it with fresh locals, including one nested in a walk of itself.
fn gen(n: Int) -> yields Int:
    var i = 0
    while i < n:
        yield i
        i += 1
var s = 0
for a in gen(3):
    for b in gen(a):
        s += a * 10 + b
print(s)
