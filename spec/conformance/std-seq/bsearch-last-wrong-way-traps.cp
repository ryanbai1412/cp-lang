// expect: R0401 at 2 (checked)
// With `last: true`, `bsearch` wants true then false; the other way traps.
print((0..10).bsearch(x => x >= 4, last: true).ord)
