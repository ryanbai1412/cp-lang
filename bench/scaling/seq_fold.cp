let n: Int = read()
let xs = [i for i in 0..n]
let add = (s: Int, x: Int) => s ^ (x + s % 7) // within: seq.fold_from
var acc = 0
for r in 0..64:
    acc += xs.fold_from(r, add) // timed: seq.fold_from
print(acc)
