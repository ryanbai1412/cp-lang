// `..xs: Ts` takes the remaining positional arguments as one tuple; `fields` walks it (decision 28.3).
fn show_line<Ts>(..xs: Ts):
    var n = 0
    for x in fields(xs):
        n += 1
        print(n, x)

fn count<Ts>(..xs: Ts) -> Int:
    var n = 0
    for _ in fields(xs):
        n += 1
    n

fn labelled<Ts>(tag: Str, ..xs: Ts):
    for x in fields(xs):
        print(tag, x)

show_line(3, "abc", 'z', true)
print(count(), count(1), count(1, 2.5, "x"))
labelled("a", 10, 20)
