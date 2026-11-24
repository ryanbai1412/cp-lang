// here: 4
// C++'s `string::npos` (self-run mistakes): the fix replaced `::` with `:` and left an error.
let s = "abc"
if s.find("b") != string::npos: print(1)
