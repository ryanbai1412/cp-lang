enum E:
    A
    B(x: 1..=3)
let e: E = read()
match e:
    A =>
        print(1)
