// `Prefix<R, C, T>.fold` is a `todo` stub: `cpl run` traps, and the C++ build does not compile.
domain R = 0..2
domain C = 0..3
let p = Prefix<R, C, Int>([R, C] (r, c) => 1 + 3 * Int(r) + Int(c))
print(p.fold(R(0)..=R(1), C(0)..=C(2)))
