// `.fold_from(x, f)` is std.seq's fold over any sequence: a list, a
// `[D]T`, a `Str` and a generator. An empty one gives `init`.
domain D = 0..3
let d: [D]Int = [D] i => Int(i) + 1
fn upto(n: Int) -> yields Int:
    for i in 1..=n: yield i
print([1, 2, 3].fold_from(0, (a, x) => a * 10 + x))
print(d.fold_from(0, (a, x) => a * 10 + x))
print("123".fold_from(0, (a, c) => a * 10 + (Int(c) - Int('0'))))
print(upto(3).fold_from(0, (a, x) => a * 10 + x))
let e: List<Int> = []
print(e.fold_from(7, (a, x) => a + x), [3, 1, 2].fold(+))
