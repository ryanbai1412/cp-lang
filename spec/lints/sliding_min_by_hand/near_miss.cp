let n: 1..=200000 = read()
let a: [0..n]Int = read()
var st: List<Int> = []
var prev: List<Int> = []
for i in 0..n:
    while st.len > 0 and a[st[st.len - 1]] >= a[i]:
        st.pop()
    prev.push(if st.len > 0: st[st.len - 1] else: -1)
    st.push(i)
print(prev)
