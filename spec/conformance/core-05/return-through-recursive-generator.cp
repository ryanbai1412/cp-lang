// expect: ok, output "7"
// rule: 05.9.3 06.3.3 07.1.1 07.6.1
fn walk(n: Int) -> yields Int:
    yield n
    if n > 0:
        for x in walk(n - 1): yield x
fn f() -> Int:
    for x in walk(10):
        if x == 7: return x
    0
print(f())
