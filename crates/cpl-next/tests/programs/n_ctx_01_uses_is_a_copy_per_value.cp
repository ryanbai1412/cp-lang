// expect: ok, output "1,2\n1 2\n10 20\n2\n7;7"
// A `uses` fn is checked once per context value a call gives it (R09.1.4); a fn without `uses`
// reads the contexts around its own declaration, which are the defaults here (R09.1.3).
context opt:
    k: Int = 2
fn pick() -> Int uses opt = opt.k
fn plain(a: Int, b: Int) = print(a, b)
fn follows(a: Int, b: Int) uses io = print(a, b)
fn both() uses io, opt = print(opt.k, opt.k)
with io(sep: ","):
    follows(1, 2)
    plain(1, 2)
with opt(k: 10):
    print(pick(), opt.k * 2)
print(pick())
with io(sep: ";"):
    with opt(k: 7):
        both()
