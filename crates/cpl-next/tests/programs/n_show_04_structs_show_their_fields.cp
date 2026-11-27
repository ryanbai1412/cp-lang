// expect: ok, output "1 2\n1 2\n3 4\nhi 5 6 7 8 9\n1 2 3\n1 2!\nP"
// rule: R10.3.5 R08.1.6
// A record or struct whose fields are all `Show` prints its fields joined by spaces, a sequence
// of them one per line, unless it writes its own impl.
struct P:
    a: Int
    b: Int
struct Q<T>:
    x: T
    p: P
struct W:
    n: Int
impl Show for W:
    fn show(self) -> Str = "P"
print(P(1, 2))
print([P(1, 2), P(3, 4)])
print(Q("hi", P(5, 6)), Q(7, P(8, 9)))
let r = (u: 1, v: [2, 3])
print(r)
print("{P(1, 2)}!")
print(W(3))
