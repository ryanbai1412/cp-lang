// A divisor whose declared range excludes 0 cannot trap.
let (a, b): (1..=100, 1..=5) = read()
print(a / b)
print(a % (b + 1))
