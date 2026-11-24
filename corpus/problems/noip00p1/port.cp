// Source: NOIP 2000 Senior P1 (Base Conversion), https://dmoj.ca/problem/noip00p1
// limits: 1000 ms, 125 MB
// Problem: an integer x and a base -20 <= r <= -2; print x written in base r, digits 0..|r|-1
//   with A, B, ... for digits over 9 (this judge wants just the digits).
// Solution: repeat d = x mod r, x = x div r with Euclidean division: the remainder is already a
//   digit in 0..|r|-1 whatever the sign of the base. Print the digits reversed. O(log |x|).
// original: corpus/submissions/5067849.cpp
let x: -32768..=32767 = read()
let base: -20..=-2 = read()

var digits: List<Char> = []
var left = x
while left != 0:
    let d = left % base
    digits.push(if d < 10: '0' + d else: 'A' + (d - 10))
    left /= base
print(digits.rev)
