// expect: ok, output "10203 5"
// rule: 06.3.1
// A loop body that calls a fn value which grows the list the generator walks is not fed: the
// generator walks the list as it was, whatever other fn types the program calls through.
var g = [1, 2, 3]

fn gen() -> yields Int:
    for x in g: yield x

fn bump(x: Int) -> Int:
    if x < 3: g.push(x + 10)
    x

fn show(s: Bool) -> Bool: s

fn run(f: Int -> Int, h: Bool -> Bool) -> Int:
    var t = 0
    for x in gen(): t = t * 100 + f(x)
    if h(true): t else: 0

print(run(bump, show), g.len)
