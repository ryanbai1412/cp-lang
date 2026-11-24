// Lists keep cells of one scalar kind packed; every list operation still sees values.
var xs: List<Int> = []
for i in 0..6:
    xs.push((i * 7) % 5)
xs[2] = 40
xs[3] += 1
print(xs, xs.len, xs.pop()!, xs)
print(xs.sorted(), yes(xs == [0, 2, 40, 2, 3]), yes(xs < [0, 3]))
var arcs = List<(to: Int, w: Int)>()
for i in 0..4:
    arcs.push((to: i, w: 10 - i))
arcs[1] = (to: 9, w: 9)
print(arcs, arcs[1].to + arcs[2].w, arcs.sorted())
var flags: List<Bool> = [true, false]
flags.push(true)
var word: List<Char> = ['c', 'p']
word.push('l')
var maybe: List<?Int> = [None, 3]
maybe.push(None)
for b in flags:
    print(yes(b))
print(word, maybe.len)
print(maybe, none: -1)
var nested: List<List<Int>> = [[1], []]
nested[1].push(2)
nested.push(xs)
print(nested, [x * 2 for x in xs], sum(x for x in xs))
for (i, x) in xs.items:
    if i < 2:
        print(i, x)
