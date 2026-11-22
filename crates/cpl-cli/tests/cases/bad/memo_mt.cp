domain I = 0..5
var base = 0
@memo
fn f(i: I) -> Int:
    i.ord + base

@multitest
fn solve():
    let b: 0..=9 = read()
    base = b
    print(f(I.first))
