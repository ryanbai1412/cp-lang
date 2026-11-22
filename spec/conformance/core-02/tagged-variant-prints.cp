// expect: ok, output "Q 3"
// rule: 02.7.2
enum Cmd:
    @tag("Q") Ask(x: Int)
    @tag("A") Add(x: Int)
print(Cmd.Ask(3))
