// CF 1299C Water Balance: a stack of blocks with increasing averages.
let n: 1..=1e6 = read()
let a = read<[n]Int>()
var st: List<(sum: Int, len: Int)> = []
for x in a:
    var cur = (sum: x, len: 1)
    while st.len > 0 and st.last!.sum * cur.len >= cur.sum * st.last!.len:
        let top = st.pop()!
        cur = (sum: cur.sum + top.sum, len: cur.len + top.len)
    st.push(cur)
for g in st:
    let v = F64(g.sum) / F64(g.len)
    repeat g.len: print(v, precision: 9)
