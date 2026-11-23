// expect: ok, output "3 2 2\n1 4"
// rule: 07.7.11
// `s.step(k)` is a view: its length rounds up, and its j-th element is the source's (j * k)-th.
let e = [3, 1, 2, 4, 5].step(2)
print(e.len, e.nth(1), e.k)
let f = [7, 1, 9, 4].step(3)
print(f.len - 1, f.nth(1))
