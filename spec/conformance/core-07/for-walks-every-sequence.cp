// expect: ok, output "14"
// rule: 07.2.1 07.3.1
fn g() -> yields Int:
    yield 1
var t = 0
for x in [1, 2]: t += x
for x in 1..=3: t += x
for x in g(): t += x
for x in Set([4]): t += x
print(t)
