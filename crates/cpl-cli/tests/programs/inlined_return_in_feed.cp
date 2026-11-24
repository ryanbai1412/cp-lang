// A small fn the VM inlines returns early. Inlined into a generator fed to a
// `for`, its `return` leaves the fn, not the generator.
fn pick(a: Int, b: Int) -> Int:
    if a > b: return a
    b

fn picks(k: Int) -> yields Int:
    for i in 0..k:
        yield pick(i, 2)

var t = 0
for v in picks(4):
    t += v
print(t)
print(picks(4).collect().sum())
