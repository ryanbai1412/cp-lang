// expect: E0601
// rule: 05.8.2
fn f(mut a: List<Int>, i: Int, j: Int):
    swap(mut a[i], mut a[j]) // <- here
