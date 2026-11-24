// YAC 6 P4: a sum of games with precomputed Grundy values for sizes 0..=100. Name the smallest
// move size in 1..=m that makes the xor zero, or -1.
let grundy = [0, 0, 1, 2, 2, 7, 3, 5, 4, 5, 10, 9, 9, 15, 5, 12, 14, 17, 15, 17, 17, 23, 24,
    27, 22, 34, 18, 28, 41, 29, 21, 34, 33, 41, 26, 55, 30, 42, 35, 39, 34, 56, 65, 39, 41, 47,
    45, 43, 40, 40, 29, 44, 58, 56, 52, 57, 55, 54, 55, 48, 61, 60, 60, 58, 44, 67, 67, 64, 64,
    37, 66, 64, 74, 70, 72, 47, 66, 75, 74, 73, 90, 81, 79, 80, 78, 81, 87, 95, 95, 83, 124, 92,
    93, 86, 67, 91, 97, 88, 97, 100, 103]
let n: 1..=1e6 = read()
let m: 1..=100 = read()
let x = read<[n]1..=100>().map(a => grundy[a]).fold(^)
print((1..=m).find(i => grundy[i] == x) ?? -1)
