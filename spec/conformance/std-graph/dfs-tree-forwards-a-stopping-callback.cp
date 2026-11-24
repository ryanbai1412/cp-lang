// expect: ok, output "2"
// input: "3 2\n1 2\n2 3\n"
// dfs_tree's walk is fed inside a pusher, so its callback forwards the outer one's stop:
// the C++ must build (92df8aebf typed it as never stopping).
let (n, m): (2..=10, 1..=20) = read()
domain Town = 1..=n
domain Road = 0..m
let roads: [Road](u: Town, v: Town) = read()
let (tree, tree_road) = Graph.undirected(roads).dfs_tree(Town.first!)
print(tree.pos(Town.last!))
