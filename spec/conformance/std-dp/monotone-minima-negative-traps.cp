// expect: R0401 (checked)
print(monotone_minima(-1, 2, (i, j) => Some(i + j)).len) // <- here
