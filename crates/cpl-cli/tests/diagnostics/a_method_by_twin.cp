domain V = 0..4
let xs = [3, 1, 2]
let ys: [V]Int = [5, 5, 1, 1]
print(xs.is_sorted_by(x => -x))
print(is_sorted_by_key(xs, x => x))
for r in xs.runs_by(x => x % 2):
    print(r.len)
print(V.group_by(v => ys[v]).len)
