// expect: E0401
// rule: 07.11.1
// `rotate_left` takes `mut self`, so it changes the list it is called on.
let a = [3, 1, 2]
a.rotate_left(1) // <- here
