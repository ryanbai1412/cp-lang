// `.rev` is std's `Rev<S>` over a list, a `Str` or an array keyed by a domain: its `take`/`drop`
// cut in walk order and keep the source's keys, and a `Rev`-typed parameter walks backwards.
domain P = 1..=3
let raw: [P]Int = [4, 5, 6]
print(List(raw.rev.take(2)), raw.rev.take(2).sum())
print(List(raw.rev.drop(1)), List(raw.rev.take_last(2)), List(raw.rev.drop_last(1)))
print([k for (k, _) in raw.rev.take(2).items])
let a = [1, 1, 2, 3, 3]
let r: Rev<List<Int>> = a.rev
fn walk(v: Rev<List<Int>>) -> List<Int> = [x for x in v]
print(walk(a.rev), a.rev.collect(), r.nth(1), r.keys)
print("abc".rev.collect(), [c for c in "xyz".rev])
