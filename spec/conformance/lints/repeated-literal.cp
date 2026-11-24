// expect: L0506
let a = [3, 4]
print(a.sum() % 998_244_353)
print(a[0] * a[1] % 998_244_353) // <- here
