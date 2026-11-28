// expect: E0503
// rule: 01.7.6
// has: "do:"
// A lambda's block is indented on the lines after `=>`; there is no `do:`.
let f = (x: Int) => do: // <- here
    let y = x + 1
    y * 2
print(f(3))
