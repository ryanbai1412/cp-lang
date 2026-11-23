// expect: ok, output "3 3"
// rule: 10.3.5
fn twice<T: Show>(x: T):
    print(x, x)
twice(3)
