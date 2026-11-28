// expect: ok, output "lt ge"
// engines: next
print(if (0, 0, 0, 1) < (0, 0, 1, 0): "lt" else: "ge", if (2, 0, 0, 0) < (1, 9, 9, 9): "lt" else: "ge")
