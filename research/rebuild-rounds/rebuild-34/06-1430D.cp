// CF 1430D String Deletion: work on the run lengths. Each operation deletes
// the front run; spend the forced single deletion on the nearest run (at or
// after the front) that has a spare character. With none left, two runs go
// per operation.
@!multitest
let n: 1..=2e5 = read()
let s = read<Str>()

var size = [r.len for r in s.runs()]
let m = size.len
var spare = 0
var ops = 0
var i = 0
while i < m:
    spare = max(spare, i)
    while spare < m and size[spare] < 2: spare += 1
    if spare == m:
        ops += (m - i + 1) / 2
        break
    size[spare] -= 1
    ops += 1
    i += 1
print(ops)
