// expect: ok, output "1,2\n1;2\n1,2\n2\n5\n9\n5\n2"
// rule: 09.1.1
context opt:
    k: Int = 2

fn show_k() uses opt = print(opt.k)

with io(sep: ","):
    print(1, 2)
    with io(sep: ";"):
        print(1, 2)
    print(1, 2)

show_k()
with opt(k: 5):
    show_k()
    with opt(k: 9):
        show_k()
    show_k()
show_k()
