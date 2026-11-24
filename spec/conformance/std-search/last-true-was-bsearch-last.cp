// expect: E0503
// tool: check
// has: "it is `.last_true`"
print((0..10).bsearch_last(x => x < 3) ?? -1) // <- here
