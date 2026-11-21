fn ask(kind: Char, x: Int) -> Int:
    print(kind, x)
    flush()
    read()

@! multitest
let n: Int = read()
print(0)
flush()

let n_sz = ask('I', 0)
if n_sz == 1: ask('I', (1 << n) - 1)

var v = 0
for b in (0..n).rev:
    if ask('Q', v + (1 << b)) > 0: v += 1 << b

var k = if n_sz == 1: 1 else: 2
if n_sz != 1:
    if v & (v - 1) != 0:
        if ask('I', v & (v - 1)) == 3: k = 3
    else:
        let nv = if v == 1: 3 else: v + 1
        ask('I', nv)
        if ask('Q', nv) != 1: k = 3

print("A", k, v)
flush()
