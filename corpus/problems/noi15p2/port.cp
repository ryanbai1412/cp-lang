// Source: NOI 2015 P2 (Software Package Manager), https://dmoj.ca/problem/noi15p2
// Problem: n <= 1e5 packages; package 0 is the root and every other package depends on one
//   package, forming a tree. q <= 1e5 commands `install x` (installs x and everything it depends
//   on) or `uninstall x` (removes x and everything depending on it); print how many packages each
//   command changes.
// Solution: install sets the root-to-x path, uninstall clears the subtree of x. In heavy-first
//   preorder the path is O(log n) chain ranges and the subtree one range, so a segment tree with
//   range assignment and sum counts installed packages before each assignment: O(log^2 n) per
//   command.
// Bounds: n, q: 1..=1e5; needs: 0..=n - 1; x: 0..n

let n: 1..=1e5 = read()
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

let q: 1..=1e5 = read()
for _ in 0..q:
    let (cmd, x): (Str, Pkg) = read()
    if cmd == "install": print(sum(switch(c.range, true) for c in tree.chains(Pkg.first!, x, false)))
    else: print(switch(tree.sub(x), false))
