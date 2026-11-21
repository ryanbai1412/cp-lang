@!multitest
let (k, x0): (Int, Int) = read()
var x = x0
var y = (1 << (k + 1)) - x
var ops: List<Int> = []
while x != y:
    if x > y:
        ops.push(2)
        x -= y
        y *= 2
    else:
        ops.push(1)
        y -= x
        x *= 2
print(ops.len)
if not ops.is_empty: print(ops.rev)
