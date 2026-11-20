// cp: 0.1 bea787c+dirty
@memo(hash)
fn f(u: 0..=50) -> Int:
    if u < 2: return u
    return f(u - 1) + f(u - 2)
print(f(30))
