let n: 1..=200000 = read()
let a: [0..n]Int = read()
var st: List<Int> = []
var ans: [0..n]Int = -1  // expect: monotone_stack_by_hand
for i in 0..n:
    while st.len > 0 and a[st.last!] < a[i]:  // expect: monotone_stack_by_hand
        let j = st.last!
        st.pop()
        ans[j] = i
    st.push(i)
print(ans)
