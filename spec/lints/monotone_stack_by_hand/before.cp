let n: 1..=200000 = read()
let a: [0..n]Int = read()
var st: List<Int> = []
var ans = [0..n] _ => -1
for i in 0..n:
    while st.len > 0 and a[st.last!] < a[i]:
        ans[st.pop()!] = i
    st.push(i)
print(ans)
