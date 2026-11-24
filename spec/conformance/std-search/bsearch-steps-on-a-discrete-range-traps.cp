// expect: R0401 (checked)
// rule: search-dp
// `steps:` is the real search's halving count; a discrete range finds its exact cut, so asking
// for steps there is a mistake, not an option to ignore.
print((0..100).bsearch(x => x >= 7, steps: 10).r) // <- here
