// `.fold(f, init: x)` is std.seq's fold over any sequence: a list, a
// `[D]T`, a `Str` and a generator. An empty one gives `init`.
domain D = 0..3
let d: [D]Int = [D] i => Int(i) + 1
fn upto(n: Int) -> yields Int:
    for i in 1..=n: yield i
print([1, 2, 3].fold((a, x) => a * 10 + x, init: 0))
print(d.fold((a, x) => a * 10 + x, init: 0))
print("123".fold((a, c) => a * 10 + (Int(c) - Int('0')), init: 0))
print(upto(3).fold((a, x) => a * 10 + x, init: 0))
let e: List<Int> = []
print(e.fold((a, x) => a + x, init: 7), [3, 1, 2].fold(+))
