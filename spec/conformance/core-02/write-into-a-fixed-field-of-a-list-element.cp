// expect: ok, output "5 -1"
// rule: 02.6.1
// A fixed field's binding is fixed, not its contents: `ns[0].kid[1] = v` writes into `kid`.
struct N:
    var cnt: Int
    kid: [0..2]?Int
var ns: List<N> = [N(0, [None, None])]
ns[0].kid[1] = 5
print(if let c = ns[0].kid[1]: c else: -1, if let c = ns[0].kid[0]: c else: -1)
