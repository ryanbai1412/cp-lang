// expect: E0302
// `Heap`'s one positional argument is its key function, so a list there is an error, not a heap of the list.
let a: List<Int> = [3, 1, 3]
let h = Heap(a) // <- here
