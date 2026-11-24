// expect: ok, output "1 2 3\n6\nab"
print([[1, 2], [3]].flatten().collect())
print([1, 2, 3].fold_from(0, (a, x) => a + x))
print(["a", "b"].fold_from("", (a, x) => a + x))
