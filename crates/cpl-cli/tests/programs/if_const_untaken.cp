const FAST = false
const N = 3
fn f(x: Int) -> Int:
    if const FAST:
        return x.no_such_thing
    else:
        return x + 1
if const 1 > 2:
    print(undefined_name)
if const not FAST: print(f(1))
if const true:
    print("picked")
else:
    print(also_undefined)
