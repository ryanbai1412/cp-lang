// A generic that is not a domain keeps a range element as `Int`: `scan<D, T>` over `[I]-9..=9`
// names `Monoid<T>` and its prefix sums leave the range.
// See compiler-changes/R6.
let n: 1..=9 = read()
domain I = 0..n
let a: [I]-9..=9 = read()
let pre = a.scan(+)
print(pre.prefix(I.cuts.first!), pre.prefix(I.cuts.last!))
