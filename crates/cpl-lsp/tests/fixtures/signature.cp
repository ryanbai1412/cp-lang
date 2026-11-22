// lsp: signatureHelp
fn walk(from: Int, steps: Int = 1; back: Bool = false) -> Int = from
let xs = [3, 1, 2]
print(walk(1, $02), walk(1, back: $0true))
let r = xs.reach(while: (i, j) => i < j, cyclic: $0true)
print(r)
