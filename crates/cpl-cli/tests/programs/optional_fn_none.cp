// An optional that holds a fn is tested against `none`: fns do not compare, so the
// built C++ asks only whether one is there.
struct Box:
    f: ?(Int -> Int)
    pub fn get(self, x: Int) -> Int:
        if f == none: return -1
        assert f != none
        f!(x)
let a = Box(none)
let b = Box(x => x * 2)
print(a.get(3), b.get(3))
