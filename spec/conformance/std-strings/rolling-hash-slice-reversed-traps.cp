// expect: R0401 (checked)
let h = RollingHash("abab")
print(h.slice(2, 1).h1) // <- here
