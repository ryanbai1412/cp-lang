// 2032C Trinity: keep a window a[i..j) of the sorted array whose two smallest
// sum beats its largest; j is the lower bound of a[i] + a[i + 1].
@!multitest
let n: 3..=2e5 = read()
let a = read<[n]1..=1e9>().sorted()
let keep = (0..n - 1).map(i => a.partition_point(x => x >= a[i] + a[i + 1]) - i).max()!
print(n - keep)
