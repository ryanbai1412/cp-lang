let n = read<1..=1e9>()
domain Rail = 1..=n
enum Cmd:
    @tag("I") Set(a: Rail, b: Rail, d: -1e9..=1e9)
    @tag("Q") Ask(h: 0..=1e9)
    @tag("E") End
let cmds = read_all<Cmd>(until: c => c == Cmd.End)
for c in cmds:
    match c:
        Set(a, b, d) => print("set", a, b, d)
        Ask(h) => print("ask", h)
        End => continue
