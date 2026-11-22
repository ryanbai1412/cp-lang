var total = 0

pure fn add(x: 0..=10) -> Int:
    total += x
    total

print(add(3))
