// expect: E0302
// rule: 02.4.9
// `?T` is not a `Seq` (R02.4.9): an optional walked as zero or one element would print a value or nothing.
let x: ?Int = Some(3)
for v in x: // <- here
    print(v)
