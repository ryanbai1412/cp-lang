// `Fenwick.fold` is a `todo` stub: `cpl run` traps, and the C++ build does not compile.
domain I = 1..=3
let fw = Fenwick([I] i => Int(i))
print(fw.fold(I(1)..=I(2)))
