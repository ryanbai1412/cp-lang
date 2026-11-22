// expect: E0606 at 3
// rule: 06.3.5 07.6.4
fn sq(n: Int) -> yields Int:
    for i in 1..=n: yield i * i
print(zip(sq(3), sq(3)).count(p => true))
