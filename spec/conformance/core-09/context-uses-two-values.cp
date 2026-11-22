// expect: ok, output "1,2\n1-2\n10\n20"
// rule: 09.1.3
context opt:
    k: Int = 1

fn show(a: Int, b: Int) uses io = print(a, b)
fn pick() -> Int uses opt = opt.k

with io(sep: ","):
    show(1, 2)
with io(sep: "-"):
    show(1, 2)
with opt(k: 10):
    print(pick())
with opt(k: 20):
    print(pick())
