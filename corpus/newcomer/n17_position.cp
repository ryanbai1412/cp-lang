let n: 2..=100 = read()
domain V = 1..=n
let a: [V]Int = read()
var cnt = 0
for v in V:  // expect: adjacent_by_index
    if v != V.last!:
        let w = V(v.ord + 1)  // expect: position_arithmetic
        if a[v] < a[w]:
            cnt += 1
print(cnt)
