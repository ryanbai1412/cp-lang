// expect: ok, output "30 7\n6 9\n25 0 1 2 3 4"
// rule: 07.1.3
// A generator walking another runs both only as far as the consumer pulls, and lends a `mut` argument through.
var made = 0
fn nat() -> yields Int:
    var i = 0
    while true:
        made += 1
        yield i
        i += 1
fn evens() -> yields Int:
    for x in nat(): yield x * 2
var s = 0
for x in evens():
    if x > 10: break
    s += x
print(s, made)
made = 0
print(evens().take(3).sum(), made + 6)
fn inner(mut seen: List<Int>, n: Int) -> yields Int:
    for i in 0..n:
        seen.push(i)
        yield i * 2
fn outer(mut seen: List<Int>, n: Int) -> yields Int:
    for x in inner(mut seen, n): yield x + 1
var seen: List<Int> = []
var t = 0
for x in outer(mut seen, 5): t += x
print(t, seen)
