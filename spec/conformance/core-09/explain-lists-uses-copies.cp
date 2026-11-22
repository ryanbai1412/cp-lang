// expect: ok
// rule: 09.1.4
// tool: explain
// has: "show"
fn show(a: Int, b: Int) uses io = print(a, b)

with io(sep: ","):
    show(1, 2)
with io(sep: "-"):
    show(1, 2)
