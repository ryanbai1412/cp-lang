// NOI 2015 Package Manager: package 0 is the root and every other package
// needs one earlier package. `install x` installs x and everything it
// needs; `uninstall x` removes x and everything that needs it. Print how
// many packages each command changes.
//
// install is "set the path from the root to x", uninstall is "clear the
// subtree of x". In heavy-first preorder both are a few ranges of one
// segment tree with range assignment that counts installed packages.

let n: Int = read()
domain Pkg = 0..n
domain Dep = 1..n  // the packages that need another
let needs: [Dep]Pkg = read()
var boss: [Pkg]?Pkg = None
for d in Dep: boss[Pkg(d.label)] = needs[d]
let tree = Rooted.from_parents(boss)

var installed = SegTree(+, [tree.Pre] _ => 0, act: assign)

fn switch<P>(r: Range<P>, on: Bool) -> Int:  // returns how many changed
    let had = installed.fold(r)
    installed.apply(r, Some(Int(on)))
    if on: r.len - had else: had

let q: Int = read()
for _ in 0..q:
    let (cmd, x): (Str, Pkg) = read()
    if cmd == "install": print(sum(switch(c.range, true) for c in tree.chains(Pkg.first!, x, false)))
    else: print(switch(tree.sub(x), false))
