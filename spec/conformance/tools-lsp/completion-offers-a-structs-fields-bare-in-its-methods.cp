// expect: ok
// rule: lsp.2
// tool: lsp completion (val: l|
// has: "label":"line"
// A method names its struct's fields without `self.`, so completion offers them bare.
struct Photo(line: Int, count: Int):
    fn at(self, x: Int) -> (val: Int, count: Int) = (val: l, count:)
