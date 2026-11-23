// expect: ok, output "1 5 3"
// rule: 05.8.2
// input: "1 2\n"
// With `i != j` proven, `mut c[i]` and a read of `c[j]` are apart.
let (i, j): (0..3, 0..3) = read()
var c = [1, 2, 3]
fn merge_into(mut dst: Int, src: Int):
    dst += src
if i != j:
    merge_into(mut c[i], c[j])
print(c)
