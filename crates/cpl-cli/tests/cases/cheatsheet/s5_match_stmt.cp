enum Op:
    Add(l: Int, r: Int)
    Query(l: Int)
let op = Op.Add(1, 2)
match op:
    Add(l, r) => print(l + r)
    Query(l) => print(l)
