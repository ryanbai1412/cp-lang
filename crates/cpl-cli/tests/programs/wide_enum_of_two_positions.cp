let n: 1..=1e5 = read()
domain C = 1..=n
enum Link:
    A(a: C, b: C)
    B(a: C, b: C)
match B(C.last!, C.last!):
    A(a, b) => print(a.ord + b.ord)
    B(a, b) => print(-a.ord - b.ord)
