// section 6: function types omit `fn`: `Int -> Int`, `(Int, Int) -> Int`
fn apply(f: Int -> Int, x: Int) -> Int = f(x)
fn both(g: (Int, Int) -> Int) -> Int = g(2, 3)
print(apply(x => x + 1, 4), both((a, b) => a * b))
