// expect: ok, output "1 1 1"
let a = if cmp(1, 2) == Ordering.Less: 1 else: 0
let b = if cmp("b", "a") == Ordering.Greater: 1 else: 0
let c = if cmp(3, 3) == Ordering.Equal: 1 else: 0
print(a, b, c)
