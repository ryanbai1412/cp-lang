// expect: ok, output "1,2"
// rule: 09.1.3
// copies: 1
with io(sep: ","):
    let show = (a: Int, b: Int) => print(a, b)
    show(1, 2)
