// expect: ok, output "eq ne"
// engines: next
print(if (1, 'x', true) == (1, 'x', true): "eq" else: "ne", if (1, 'x', true) == (1, 'x', false): "eq" else: "ne")
