let n: 1..=100 = read()
domain Len = 1..=n
let a: [Len]Int = read()
let st = SegTree((a, b) => a + b, a)
print(st.fold(Len.first..=Len.last))
