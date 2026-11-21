// Names come from the type asked for, in order: swapped field types don't fit.
let n = 2
var es: [n](Int, Bool) = (0, false)
let swapped: [n](a: Bool, b: Int) = es
let o: ?(Int, Bool) = none
let os: ?(a: Bool, b: Int) = o
