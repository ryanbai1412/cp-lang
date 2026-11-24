// expect: L0602
// rule: 07.1.3 07.7.2
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i
let a = [1, 2, 5, 1]
let c = a.drop_while(x => x < 3)
let d = upto(9).take_while(x => x < 4)
print(c.sum(), d.sum())
print(c.sum()) // <- here
