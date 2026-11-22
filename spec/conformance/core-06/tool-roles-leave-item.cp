// expect: ok, output "4"
// rule: 06.4.3
@gen
fn g() -> Int = 4
@test
fn t(): assert(g() == 4)
t()
print(g())
