// expect: ok, output "5 ask"
// rule: 10.2.3
enum Cmd:
    Add(x: Int)
    Ask
fn show(c: Cmd) -> Str = match c:
    Add(x) => "{x}"
    Ask => "ask"
print(show("Add 5".parse<Cmd>()!), show("Ask".parse<Cmd>()!))
