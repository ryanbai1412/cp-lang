// CSES 1732 Finding Borders
// in: a string s; out: the lengths of all borders of s, in increasing order
use std.strings.{borders}
let s = read<Str>()
print(borders(s).collect().rev)
