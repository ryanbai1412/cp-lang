let n: 1..=1e6 = read()
var x = n
var steps = 0
var peak = n
while x != 1:
    x = if x % 2 == 0: x / 2 else: 3 * x + 1
    peak max= x
    steps += 1
print(steps, peak)
