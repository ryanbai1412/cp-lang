// here: 4
// Python's `s.split(",")`: a `Str` splits on a `Char`, so a one-char `Str` separator is fixed to `','`.
let s = "a,b,c"
for part in s.split(","):
    print(part)
