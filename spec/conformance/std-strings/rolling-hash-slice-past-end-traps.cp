// expect: R0401 (checked)
let h = RollingHash("ab")
print(h.slice(0, 3).h1) // <- here
