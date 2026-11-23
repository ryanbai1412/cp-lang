// expect: ok, output "2 1 3\n2 3\n1 1 1 1\n2 2\n3"
print([r.len for r in [1, 1, 2, 3, 3, 3].runs()])
print([r.len for r in [1, 2, 4, 5, 6].runs(link: (x, y) => y == x + 1)])
print([r.len for r in [4, 3, 2, 1].runs(link: (x, y) => x < y)])
print([r.len for r in [1, 3, 2, 4].runs(by: x => x % 2)])
print([r.len for r in [5, 1, 9].runs(by: x => x % 2, link: (x, y) => true)])
