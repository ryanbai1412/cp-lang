// expect: ok, output "1,2 5"
// rule: 09.1.5
context opt:
    k: Int = 2
fn pick() -> Int uses opt = opt.k
with io(sep: ",")
with checks(bounds: on)
with lint(float_equality: allow)
with opt(k: 5):
    print("1,2 {pick()}", sep: " ")
