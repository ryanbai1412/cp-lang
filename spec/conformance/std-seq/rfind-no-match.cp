// expect: ok, output "-1 4"
print([1, 3, 5].rfind(x => x % 2 == 0) ?? -1, [2, 1, 2, 4].rfind(x => x % 2 == 0) ?? -1)
