// here: 4
// Python's `" ".join(map(str, a))` (self-run mistakes): it was told to import `map`, and `str` stayed unknown.
let a = [3, 1, 2]
print(" ".join(map(str, a)))
