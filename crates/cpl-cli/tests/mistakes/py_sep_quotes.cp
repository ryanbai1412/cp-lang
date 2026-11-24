// here: 4
// Python's `print(*a, sep=' ')` (self-run mistakes): after the `*` and `=` fixes, "expected `Str`, found `Char`", no fix.
let a = [3, 1, 2]
print(a, sep: ' ')
