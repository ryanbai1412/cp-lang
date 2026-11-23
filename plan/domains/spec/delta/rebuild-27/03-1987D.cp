// CF 1987D: Alice eats the smallest value she can; Bob erases whole value groups.
// del[i, j]: fewest Bob moves to erase j of the first i groups in time (None: impossible).
@!multitest
let n: 1..=5000 = read()
let a = read<[n]1..=n>()

var cnt: [1..=n]Int = 0
for v in a: cnt[v] += 1
let c = [cnt[v] for v in 1..=n if cnt[v] > 0]
let k = c.len

var del: [0..=k, 0..=k]?Int = None
del[0, 0] = 0
for i in 1..=k, j in 0..=i:
    del[i, j] = del[i - 1, j]
    if j > 0:
        if let Some(used) = del[i - 1, j - 1]:
            if used + c[i - 1] <= (i - 1) - (j - 1): del[i, j] min= used + c[i - 1]

let erased = (0..=k).last_true(j => del[k, j] != None)!
print(k - erased)
