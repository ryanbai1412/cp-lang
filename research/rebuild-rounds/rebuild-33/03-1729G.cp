// cf1729g Cut Substrings: occurrences from the Z-function; dp over the first occurrence still
// alive: cut one occurrence that overlaps it, then jump to the first occurrence past the cut.
type Z = Mod<1_000_000_007>

@!multitest
let s: Str = read()
let t: Str = read()
let l = t.len
let z = z_function(t + "#" + s)
let occ = [i for i in 0..s.len if z[l + 1 + i] >= l]
let k = occ.len

var ops: [0..=k]Int = 0
var ways: [0..=k]Z = Z(1)
for a in (0..k).rev:
    var best: ?Int = None
    var cnt = Z(0)
    for c in a..k:
        if occ[c] >= occ[a] + l: break
        var nx = c
        while nx < k and occ[nx] < occ[c] + l: nx += 1
        let cand = 1 + ops[nx]
        if best == None or cand < best!:
            best = Some(cand)
            cnt = ways[nx]
        elif cand == best!: cnt += ways[nx]
    ops[a] = best!
    ways[a] = cnt
print(ops[0], ways[0])
