// expect: ok, output "3 3\n5 0 3 0 1 0 1 0 1 2"
// z_function and prefix_function key their result by the sequence's own keys, here a declared domain.
domain K = 5..10
let s: [K]Char = [K] k => "ababa"[k.ord]
print(z_function(s)[K(7)], prefix_function(s)[K(9)])
print(z_function("ababa"), prefix_function("aabaa"))
