// expect: E0601
// rule: 05.8.2 05.8.3
// A call is one access to its arguments (R10): `mut c[i]` with a read of `c[j]` conflicts unless `i != j` is proven (acceptance design-mut-read-overlap-accepted).
let (i, j): (0..3, 0..3) = read()
var c = [1, 2, 3]
fn merge_into(mut dst: Int, src: Int):
    dst += src
merge_into(mut c[i], c[j]) // <- here
print(c)
