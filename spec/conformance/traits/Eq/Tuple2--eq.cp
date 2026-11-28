// expect: ok, output "eq ne"
// engines: next
print(if (1, "a") == (1, "a"): "eq" else: "ne", if (1, "a") == (1, "b"): "eq" else: "ne")
