// expect: ok, output "c 1"
// rule: 02.2.1
// `Char` is ordered by code, so `max` and `<` work (rebuild round 17 H08).
print(max('a', 'c'), if 'b' < 'c': 1 else: 0)
