let n: 1..=100 = read()
domain Len = 1..=n
let a: [Len]Int = read()
let st = SegTree((a, b) => a + b, a)
let x = Len.first
print(st.fold(x..Len.last))
