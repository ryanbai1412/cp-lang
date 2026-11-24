// A `yields` fn fed by an endless `yields` fn, walked by a loop that breaks:
// a profiled run feeds each level too, so it stops as a plain run does.
fn nat() -> yields Int:
    var i = 0
    while true:
        yield i
        i += 1

fn squares() -> yields Int:
    for x in nat(): yield x * x

for v in squares():
    if v > 50: break
    print(v)
