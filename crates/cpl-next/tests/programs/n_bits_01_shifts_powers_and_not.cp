// expect: ok, output "8 2 -6 -4 -1 1 4611686018427387904"
// rule: R01.5.8
// Shifts, `~` and `**` on `Int`: `>>` keeps the sign, and a huge power of -1 is still exact.
let big = 1000000000001
print(1 << 3, 9 >> 2, ~5, -7 >> 1, (-1) ** big, 0 ** 0, 2 ** 62)
