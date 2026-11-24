let xs: List<Int> = [1, 2, 3]
var total = 0
for x in xs:
    total += x
var odd = 0
for x in 1..10 by 2:
    odd += x
print(total, odd)
