// `sort(by:)` and `sorted(by:)` keep equal keys in order, call `by` once per
// element, and give the same order on every receiver and in a generic body.
fn by_mod<S: IndexedMut>(mut a: S):
    a.sort(by: p => p.0 % 3)
fn firsts<S: Seq<(Int, Int)>>(s: S) -> List<(Int, Int)> = s.sorted(by: p => desc(p.0 % 4))
let n: 1..=1000 = read()
let xs = [(i * 7 % 10, i) for i in 0..n]
var calls = 0
let s = xs.sorted(by: p => { calls += 1; p.0 % 5 })
print(calls, s)
print(firsts(xs))
var a = xs.clone()
by_mod(mut a)
print(a)
var g: [0..n](Int, Int) = [xs[i] for i in 0..n]
by_mod(mut g)
print([g[i] for i in 0..n])
let words = ["bb", "a", "ccc", "dd", "e", "fff"]
print(words.sorted(by: w => w.len), words.sorted(by: w => desc(w.len)))
