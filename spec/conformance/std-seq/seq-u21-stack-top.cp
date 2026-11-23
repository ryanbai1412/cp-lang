// expect: ok, output "7 7\n1"
// Acceptance for seq/U21 (stack top, push, pop), handbook/product/std/sequences-usecases.md.
var st = [1, 3, 5]
st.push(7)
print(st.last!, st.pop()!)
st.pop_while(x => x > 2)
print(st)
