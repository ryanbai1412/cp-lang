let n: 1..=90 = read()
var ways: [0..=90]Int = 0
ways[0] = 1
for i in 1..=n:
    ways[i] = ways[i - 1] + (if i >= 2: ways[i - 2] else: 0)
print(ways[n])
