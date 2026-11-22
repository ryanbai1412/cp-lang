// expect: ok, output "1,2"
// rule: 09.1.3
with io(sep: ","):
    let show = (a: Int, b: Int) => print(a, b)
    show(1, 2)
