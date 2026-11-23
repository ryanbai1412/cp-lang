let a = [3, 9, 2]
for (i, x) in (0..3).items:
    print(i, x)
for (i, x) in (2..5).rev.items:
    print(i, x)
print([i * x for (i, x) in a.map(x => x + 1).items])
fn f(n: Int) -> List<Int> = [k * k for k in 0..n]
var calls = 0
fn g(n: Int) -> List<Int>:
    calls += 1
    f(n)
for (i, x) in g(4).items:
    print(i, x)
print(calls)
print([c for (i, c) in "abc".to_upper().items if i != 1])
