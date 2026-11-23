// Codeforces 1914D: Three Activities
// I/O: t tests; each n then arrays a, b, c of n friends per day; print max a_x + b_y + c_z over distinct days x, y, z.
@!multitest
let n: 3..=1e5 = read()
domain Day = 0..n
let a = read<[Day]1..=1e8>()
let b = read<[Day]1..=1e8>()
let c = read<[Day]1..=1e8>()
// an optimum uses one of the three best days of each activity
fn top3(x: [Day]1..=1e8) -> List<Day> = Day.max_k(3, by: d => x[d])
var best = 0
for i in top3(a):
    for j in top3(b):
        for k in top3(c):
            if i != j and j != k and i != k: best max= a[i] + b[j] + c[k]
print(best)
