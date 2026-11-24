// expect: E0408
// rule: 07.2.5
// A range is positions with no data behind it, so there is nothing to write into.
var r = 0..5
r[1] = 3 // <- here
print(r.len)
