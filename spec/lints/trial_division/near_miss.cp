let x: 1..=1000000000 = read()
var d = 1
while d * d <= x:
    d += 1
print(d - 1)
