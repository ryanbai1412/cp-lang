// cp: 0.1 38a7ef4+dirty
// A domain's positions by label (`at`) and by offset (`nth`); the mirror of `i` is the offset counted
// from the far end.
let n: 1..=9 = read()
domain I = 1..=n
let a: [I]0..=100 = read()
let b = [I] i => a[I.nth(I.count - 1 - i.ord)]
print(I.at(1)!.ord, I.at(n)!.ord, I.nth(0).ord, b)
