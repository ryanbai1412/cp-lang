var a: List<Int> = [3, 1, 2]
a.sort(by: x => a[0] - x)
print(a[0])
