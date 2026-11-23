// Codeforces 1983C Have Your Cake and Eat It Too
// in: t; per case n, then three lines of n values (Alice, Bob, Charlie). out: per case la ra lb rb lc rc (1-based) or -1
@!multitest
let n = read<3..=2e5>()
let v = [read<[n]1..=1e6>(), read<[n]1..=1e6>(), read<[n]1..=1e6>()]
let need = v[0].sum().ceil_div(3)
for order in [0, 1, 2].permutations():
    var slice = [(0, 0), (0, 0), (0, 0)]
    var i = 0
    var ok = true
    for p in order:
        let l = i
        var got = 0
        while i < n and got < need:
            got += v[p][i]
            i += 1
        if got < need:
            ok = false
            break
        slice[p] = (l + 1, i)
    if ok:
        let last = order[2]
        slice[last] = (slice[last].0, n)
        print(slice)
        break
else:
    print(-1)
