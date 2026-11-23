// expect: ok, output "YES YES 3\nYES NO 4"
domain V = 0..5
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]
let t = Rooted(adj, root: 0)
let (domain S, of) = t.shapes()
print(yes(of[3] == of[2]), yes(of[1] != of[0]), S.count)
domain W = 0..6
let path: [W]List<W> = [[1, 3], [0, 2], [1], [0, 4], [3, 5], [4]]
let p = Rooted(path, root: 0)
let (domain T, sh) = p.shapes()
print(yes(sh[2] == sh[5]), yes(sh[1] == sh[3]), T.count)
