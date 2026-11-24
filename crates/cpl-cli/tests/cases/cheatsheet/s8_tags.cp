enum Cell: @tag('.') Empty | @tag('*') Wall | @tag('1'..='9') Coin(val: 1..=9)
let a: [0..3]Cell = read()
match a[2]:
    Coin(v) => print(v)
    Empty => print(0)
    Wall => print(-1)
