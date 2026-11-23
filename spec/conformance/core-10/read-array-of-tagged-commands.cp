// expect: ok, output "add 5\nask 7"
// input: "1 5 2 7\n"
// rule: 10.2.3
// `read<[q]Cmd>()` reads q commands, each tag then its payload.
enum Q = @tag(1) Add(x: Int) | @tag(2) Ask(x: Int)
for q in read<[2]Q>():
    match q:
        Add(x) => print("add", x)
        Ask(x) => print("ask", x)
