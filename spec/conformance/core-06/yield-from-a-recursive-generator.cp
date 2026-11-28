// expect: ok, output "1\n2\n3\n4\n6"
// rule: 06.3.6
// `yield from` a recursive call yields each of its values in order (R06.3.3).
fn upto(n: Int) -> yields Int:
    if n > 0:
        yield from upto(n - 1)
        yield n
for x in upto(4): print(x)
print(upto(3).sum())
