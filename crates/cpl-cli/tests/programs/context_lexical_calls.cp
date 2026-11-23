context opt:
    k: Int = 1
fn lexical() -> Int = opt.k
fn follows() -> Int uses opt = opt.k
fn passes() -> Int = follows()
fn chain() -> Int uses opt = follows()
fn inner(a: Int) uses io = print(a, a)
fn outer(a: Int) = inner(a)
fn relay(a: Int) uses io = inner(a)
fn local() -> Int:
    with opt(k: 3):
        return lexical() + follows() * 10 + opt.k * 100
with opt(k: 10):
    print(lexical(), follows(), passes(), chain(), local())
with io(sep: ","):
    outer(1)
    relay(2)
    inner(3)
