enum Op:
    Add(l: Int, r: Int)
    Query(l: Int)
let op = Op.Add(1, 2)
let res = match op:
    Add(l, r) => l + r
    Query(l) => l
print(res)
