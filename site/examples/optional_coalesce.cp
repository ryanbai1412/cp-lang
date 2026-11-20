let n: 1..=100 = read()
let a: [0..n]Int = read()

var best_even: ?Int = none
var first_neg: ?Int = none
for x in a:
    if x % 2 == 0:
        best_even max= x
    if x < 0:
        first_neg ??= x
print(best_even ?? -1, first_neg ?? 0)
