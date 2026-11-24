// expect: L0602
// rule: 07.1.3 07.7.2
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i
let d = upto(9).take_while(x => x < 4)
print(d.sum())
print(d.sum()) // <- here
