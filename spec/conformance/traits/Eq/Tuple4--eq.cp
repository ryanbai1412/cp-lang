// expect: ok, output "eq ne"
// engines: next
print(if (1, 2, 3, 4) == (1, 2, 3, 4): "eq" else: "ne", if (1, 2, 3, 4) == (1, 2, 3, 5): "eq" else: "ne")
