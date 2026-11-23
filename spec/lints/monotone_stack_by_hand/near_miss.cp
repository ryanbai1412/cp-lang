let n: 1..=200000 = read()
let a: [0..n]Int = read()
var st: List<Int> = []
for x in a:
    while st.len > 0 and st.last! > x:
        st.pop()
    st.push(x)
print(st)
// A stack of positions whose pops do more than record an answer (here, link a tree) is not a
// nearest-greater query.
var parent = [0..n] _ => -1
var at: List<Int> = []
for i in 0..n:
    var last = -1
    while at.len > 0 and a[at.last!] < a[i]:
        last = at.pop()!
    if last >= 0:
        parent[last] = i
    at.push(i)
print(parent)
