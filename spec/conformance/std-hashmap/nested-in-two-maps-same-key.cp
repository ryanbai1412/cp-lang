// expect: ok, output "10 20"
// `m[k]` under `if k in m:` reads m's own entry, even when an enclosing `if k in other:` tested the same key on another map.
var a: Map<Int, Int> = Map()
var b: Map<Int, Int> = Map()
a[1] = 10
b[1] = 20
let k = 1
if k in a:
    if k in b:
        print(a[k], b[k])
