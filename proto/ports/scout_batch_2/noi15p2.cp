// NOI 2015 Package Manager: package 0 is the root and every other package
// needs one earlier package. `install x` installs x and everything it
// needs; `uninstall x` removes x and everything that needs it. Print how
// many packages each command changes.
//
// install is "set the path from the root to x", uninstall is "clear the
// subtree of x". In heavy-first preorder both are a few ranges of one
// lazy tree that counts installed packages.
use std.algebra.{Action}
use std.lazy.{LazySegTree, SumLen}
use std.tree.{Rooted}

let n: Int = read()
domain Pkg = 0..n
domain Dep = 1..n                           // the packages that need another
let needs: [Dep]Pkg = read()
var needed_by: [Pkg]List<(to: Pkg)> = []
for d in Dep: needed_by[needs[d]].push((to: Pkg(d.label)))
let tree = Rooted<Pkg>(u => needed_by[u], Pkg.first)

type Pre = Rooted<Pkg>.Pre
// a node is (installed, size); an update sets a whole range on or off
let set_all = Action<(Int, Int), ?Bool>(none, (f, g) => f ?? g, (f, x) => if f == none: x else: (if f!: x.1 else: 0, x.1))
var installed = LazySegTree<Pre, (Int, Int), ?Bool>.from(SumLen, set_all, [Pre] p => (0, 1))

fn switch(r: Range<Pre>, on: Bool) -> Int:  // returns how many changed
    let (had, size) = installed.fold(r)
    installed.apply(r, on)
    if on: size - had else: had

let q: Int = read()
for _ in 0..q:
    let (cmd, x): (Str, Pkg) = read()
    if cmd == "install": print(sum(switch(c.range, true) for c in tree.chains(Pkg.first, x, false)))
    else: print(switch(tree.sub(x), false))
