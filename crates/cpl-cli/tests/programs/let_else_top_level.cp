// A top-level `let v = x else: ...` narrows `v` to the value, as in a fn body.
let x: ?Int = if 2 > 1: Some(4) else: None
let y = x else:
    print("none")
    return
print(y + 1)
let xs: ?List<Int> = Some([1, 2])
let ys = xs else: return
print(ys.len)
for i in 0..3:
    let j = (if i == 1: None else: Some(i * 10)) else: continue
    print(j + 1)
