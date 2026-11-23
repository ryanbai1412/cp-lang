// expect: ok, output "[1, 5, 3]"
// rule: 05.8.2
fn merge_into(mut dst: Int, src: Int): dst += src
var c = [1, 2, 3]
let (i, j) = (1, 2)
if i != j: merge_into(mut c[i], c[j])
print(c)
