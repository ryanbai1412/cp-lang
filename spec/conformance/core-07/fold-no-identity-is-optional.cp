// expect: E0706 at 2
// rule: 07.9.2
let a = [3, 1, 2]
let x: Int = a.fold((x, y) => x * 10 + y)
