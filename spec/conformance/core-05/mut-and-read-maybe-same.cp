// expect: E0601
// rule: 05.8.2
fn merge_into(mut dst: Int, src: Int): dst += src
fn f(mut c: List<Int>, i: Int, j: Int): merge_into(mut c[i], c[j]) // <- here
