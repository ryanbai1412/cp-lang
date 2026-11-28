// expect: ok, output "lt ge"
// engines: next
print(if (1, 2, 9) < (1, 3, 0): "lt" else: "ge", if (1, 2, 3) < (1, 2, 3): "lt" else: "ge")
