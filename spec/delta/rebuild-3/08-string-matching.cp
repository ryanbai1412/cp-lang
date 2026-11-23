// CSES 1753 String Matching: count the (possibly overlapping) occurrences of a pattern in a string.
// In: two lines, the string and the pattern. Out: the number of occurrences.
let s = read<Str>()
let p = read<Str>()
print(s.find_all(p).count())
