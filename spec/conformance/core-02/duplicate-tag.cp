// expect: E0407 at 3
// rule: 02.7.2
enum Cmd:
    @tag("Q") Ask(x: Int)
    @tag("Q") Add(x: Int)
