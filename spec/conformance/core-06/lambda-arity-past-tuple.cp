// expect: E0303 at 1
// rule: 06.1.4
print([(1, 2), (3, 4)].map((a, b, c) => a + b + c).collect())
