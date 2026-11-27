// expect: E0201
// rule: diagnostics
// lacks: "unknown type"
// lacks: "slice:"
// A pattern on a value already diagnosed binds its names, domains too, to that error: their uses
// say nothing more.
if let (domain D, c) = nope: // <- here
    let o: ?D = None
    print(D.len + c)
