var p = 12
var steps = 0
while p > 0:
    p -= p & -p
    steps += 1
print(steps)
