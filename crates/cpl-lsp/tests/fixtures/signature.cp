// lsp: signatureHelp
fn walk(from: Int, steps: Int = 1; back: Bool = false) -> Int = from
let xs = [3, 1, 2]
print(walk(1, $02), walk(1, back: $0true))
let r = xs.reach(while: (i, j) => i < j, cyclic: $0true)
print(r)
var b = [3, 1, 2]
b.sort(by: $0x => x % 2)
let s = "aab"
let top = b.max($0)
print(s.runs(by: $0c => c).count(), top ?? 0)
print([3, 1, 4].lis_ends(strict: $0false))
