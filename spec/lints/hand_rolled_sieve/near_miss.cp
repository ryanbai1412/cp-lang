let n = 20
var odd = List.filled(n + 1, false)
for i in 0..=n:
    odd[i] = i % 2 == 1
print(yes(odd[7]))
