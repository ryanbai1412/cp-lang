// expect: ok, output "3 3 2 1 1\n3 3 2 1 1\n3 3 2\ncba\n4"
// rule: 07.3.5
// `.rev` has the type `Rev<S>`, which a program can name for a binding or a parameter; it walks backwards the same through `for`, `collect` and a `Str`.
let a = [1, 1, 2, 3, 3]
let r: Rev<List<Int>> = a.rev
print([x for x in r])
print(a.rev.collect())
fn firsts(v: Rev<List<Int>>) -> List<Int> = [k for (k, _) in v.items].take(3).collect()
print([a[k] for k in firsts(a.rev)])
print("abc".rev.collect())
print(r.nth(1) + 1)
