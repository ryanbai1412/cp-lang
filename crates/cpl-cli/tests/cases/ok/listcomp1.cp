// cp: 0.1 e133df2+dirty
let n: 1..=10 = read()
domain I = 1..=n
let a: [I]-100..=100 = read()
let pos = [i for i in I if a[i] > 0]
let sq: List<Int> = [a[i] * a[i] for i in I]
let pairs = [(i, j) for i in I for j in I if i < j and a[i] + a[j] == 0]
print(pos.len, sq.len, pairs.len)
for (i, j) in pairs:
    print(i, j)
