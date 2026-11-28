// expect: E0310
// rule: search-dp
// `steps:` is the real search's halving count, a parameter of `RealRange.bsearch` only; a
// discrete range finds its exact cut, so asking for steps there is a mistake the type catches.
print((0..100).bsearch(x => x >= 7, steps: 10).r) // <- here
