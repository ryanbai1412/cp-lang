// expect: E0606
// rule: R04.8.11
// `==` on `[D]T` is `List`'s at `D = Int`, which needs `T: Eq`.
fn f<D: Discrete, T>(a: [D]T, b: [D]T) -> Int = if a == b: 1 else: 0
print(f<Int, Int>([4, 5], [4, 5]))
