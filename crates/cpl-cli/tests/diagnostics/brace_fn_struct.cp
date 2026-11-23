struct P {
    x: Int
}
fn f(p: P) -> Int {
    return p.x + 1
}
print(f(P(x: 1)))
