// expect: ok, output "7 4"
// rule: 13.3.3
fn odd(n: Int) -> yields Int:
    if n > 0:
        yield n
        for x in even(n - 1): yield x
fn even(n: Int) -> yields Int:
    if n > 0:
        yield -n
        for x in odd(n - 1): yield x
fn twice(n: Int) -> yields Int:
    for x in odd(n): yield 2 * x
var s = 0
for x in odd(9):
    if x == -4: break
    s += x
var t = 0
for x in twice(4): t += x
print(s, t)
