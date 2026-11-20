fn do_query(kind: Char, v: Int) -> Int:
    print(kind, v)
    flush()
    read()

@!multitest
let n: Int = read()
print(0)
flush()
let n_sz = do_query('I', 0)
if n_sz == 1:
    let _ = do_query('I', (1 << n) - 1)
    var v = 0
    for b in (0..n).rev:
        let cnd = v + (1 << b)
        if do_query('Q', cnd) > 0:
            v = cnd
    print("A", 1, v)
    flush()
else:
    var v = 0
    for b in (0..n).rev:
        let cnd = v + (1 << b)
        if do_query('Q', cnd) > 0:
            v = cnd
    var k = 2
    if (v & (v - 1)) != 0:
        if do_query('I', v & (v - 1)) == 3:
            k = 3
    else:
        let nv = if v == 1: v + 2 else: v + 1
        let _ = do_query('I', nv)
        if do_query('Q', nv) != 1:
            k = 3
    print("A", k, v)
    flush()
