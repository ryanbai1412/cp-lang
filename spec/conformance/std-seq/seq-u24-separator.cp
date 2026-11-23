// expect: ok, output "1, 2, 3\n1, 2, 3\nab\n8-10"
// Acceptance for seq/U24 (print or join with a separator), handbook/product/std/sequences-usecases.md.
print([1, 2, 3], sep: ", ")
print(", ".join([1, 2, 3]))
print("".join(["a", "b"]))
print([4, 5].map(x => x * 2), sep: "-")
