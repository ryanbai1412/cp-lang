// expect: ok, output "1 2 3\n6\nab"
print([[1, 2], [3]].flatten().collect())
print([1, 2, 3].fold((a, x) => a + x, 0))
print(["a", "b"].fold((a, x) => a + x, ""))
