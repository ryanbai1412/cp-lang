// expect: ok, output "YES"
// `Functional(nx)` calls `Functional.new`; every query agrees with walking `nx`.
var ok = true
for trial in 0..40:
    domain V = 0..8
    let nx: [V]V = [V] _ => V(rand(0, 7))
    let f = Functional(nx)
    for v in V:
        var x = v
        for _ in 0..8: x = nx[x]
        let c = x
        var len = 1
        var y = nx[c]
        while y != c:
            y = nx[y]
            len += 1
        var on = false
        var z = c
        for _ in 0..len:
            if z == v: on = true
            z = nx[z]
        var d = 0
        var w = v
        var w_on = on
        while not w_on:
            w = nx[w]
            d += 1
            var t = c
            for _ in 0..len:
                if t == w: w_on = true
                t = nx[t]
        if f.on_cycle(v) != on: ok = false
        if f.depth(v) != d: ok = false
        if f.root_of(v) != w: ok = false
        if f.cycle_len(v) != len: ok = false
        var k = v
        for s in 0..20:
            if f.after(v, s) != k: ok = false
            k = nx[k]
print(if ok: "YES" else: "NO")
