var stk = [1, 2, 3]
var sum = 0
for _ in 0..2:
    let u = stk.last!
    stk.pop()
    sum += u
print(sum, stk.len)
