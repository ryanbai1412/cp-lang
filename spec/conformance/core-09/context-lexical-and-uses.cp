// expect: ok, output "1 2\n1,2"
// rule: 09.1.3
fn plain(a: Int, b: Int) = print(a, b)
fn follows(a: Int, b: Int) uses io = print(a, b)
with io(sep: ","):
    plain(1, 2)
    follows(1, 2)
