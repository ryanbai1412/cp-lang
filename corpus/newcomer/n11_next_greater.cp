let n: 1..=200000 = read()
let a: [0..n]Int = read()
var st: List<Int> = []
var ans: [0..n]Int = -1  // expect: missing sentinel-instead-of-optional
for i in 0..n:
    while st.len > 0 and a[st.last!] < a[i]:  // expect: missing manual-monotone-stack
        let j = st.last!  // expect: peek_then_pop
        st.pop()
        ans[j] = i
    st.push(i)
print(ans)
