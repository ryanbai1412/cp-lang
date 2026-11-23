// cp: 0.1 2fe0c49+dirty
let n: 1..=10 = read()
domain I = 1..=n

enum Op:
    Add(x: Int)
    Neg

var plan: [I]?Op = None
for i in I:
    if i.idx % 3 == 0: plan[i] = Add(i.idx)
    elif i.idx % 3 == 1: plan[i] = Neg
var v = 0
for i in I:
    if let op = plan[i]:
        match op:
            Add(x) => v += x
            Neg => v = -v
print(v, count(plan[i] == None for i in I))
