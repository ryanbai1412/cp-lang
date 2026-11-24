// expect: E0902
// rule: 03.1.10
print(repeat("ab", 2_147_483_648).len) // <- here
