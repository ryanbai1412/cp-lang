var n: 1..=1e18 = read()
var halvings = 0
while n % 2 == 0:
    n = n / 2
    halvings += 1
print(halvings, n)
