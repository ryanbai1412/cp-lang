// expect: ok, output "1 2\n1,2"
// rule: 09.1.3
fn show(xs: List<Int>):
    print(xs)
with io(sep: ","):
    show([1, 2])
    print([1, 2])
