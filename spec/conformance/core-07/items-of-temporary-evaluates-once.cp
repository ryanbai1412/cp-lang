// expect: ok, output "36 1\n2"
// rule: 07.3.2
// `.items` on a temporary evaluates it once and walks its keys like a place's.
var calls = 0
fn g(n: Int) -> List<Int>:
    calls += 1
    [k * k for k in 0..n]
var s = 0
for (i, x) in g(4).items:
    s += i * x
print(s, calls)
print([c for (i, c) in "abc".to_upper().items if i != 1].len)
