// expect: ok, output "YES YES"
// rule: 07.11.3
let g = [10, 20, 30, 21, 11].group(by: x => x % 10)
let h = [(2, 'x'), (1, 'y'), (2, 'z')].group(by: p => p.0, of: p => p.1)
print(yes(g[0] == [10, 20, 30] and g[1] == [21, 11]), yes(h[1] == ['y'] and h[2] == ['x', 'z']))
