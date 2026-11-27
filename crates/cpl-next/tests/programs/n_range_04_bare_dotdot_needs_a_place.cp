// expect: E0302
// A bare `..` takes its domain from its place, an index or a `Range<D>` slot; a `let` gives it none (R04.8.6).
let xs = [1, 2, 3]
print(xs[..].len)
let r = .. // <- here
