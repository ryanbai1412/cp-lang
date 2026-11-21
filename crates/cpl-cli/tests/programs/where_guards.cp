let xs = [1, 2, 3, 4, 5]
let s = sum(x for x in xs if x % 2 == 1)
print(s)

let evens = [x * 10 for x in xs if x % 2 == 0]
print(evens)

let a: ?Int = 5
let b: ?Int = 6
let c: ?Int = none
if let x = a and x > 3:
    print(x)
if let x = a, y = b and x + y == 11:
    print(x * y)
if let x = a, y = c:
    print(x + y)
else:
    print(-1)

var cur: ?Int = 3
var sum_w = 0
while let v = cur and v > 0:
    sum_w += v
    cur = if v > 1: v - 1 else: none
print(sum_w)
