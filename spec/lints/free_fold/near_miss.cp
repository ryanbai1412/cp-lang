let xs = [3, 1, 4]
print(sum(x * x for x in xs), min(xs[0], xs[1]), count(x > 2 for x in xs))
