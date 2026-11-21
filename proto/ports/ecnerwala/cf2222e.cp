// Codeforces 2222 E (interactive): binary-search the hidden mask v bit by bit,
// then distinguish operation k in {1, 2, 3} with at most two extra queries.
@!interactive

fn ask(kind: Char, x: Int) -> Int:
    print(kind, x)
    read()

@!multitest
let n: Int = read()
print(0)

let n_sz = ask('I', 0)
if n_sz == 1: ask('I', (1 << n) - 1)

var v = 0
for b in (0..n).rev:
    if ask('Q', v + (1 << b)) > 0: v += 1 << b

let k = if n_sz == 1:
    1
elif not v.is_pow2:
    if ask('I', v & (v - 1)) == 3: 3 else: 2
else:
    let nv = if v == 1: 3 else: v + 1
    ask('I', nv)
    if ask('Q', nv) != 1: 3 else: 2

print("A", k, v)
