// Set, in the forms proto/cpc builds today. See ../coci21c1p4.cp.
// The record of two Ints is two arrays, the stepped range is a `while`, and
// the cube sum is a plain Int (it is at most 3^k * n^2 < 2^58).
// original: corpus/submissions/4106618.cpp
let n: Int = read()
let k: Int = read()
var size = 1
for _ in 0..k: size *= 3
var re: [size]Int = 0
var w: [size]Int = 0
for _ in 0..n:
    let card: [k]Char = read()
    var code = 0
    for i in 0..k: code = code * 3 + (card[i] - '1')
    re[code] += 1

var stride = 1
while stride < size:
    var base = 0
    while base < size:
        for i in base..base + stride:
            let j = i + stride
            let l = j + stride
            let xr = re[i]
            let xw = w[i]
            let yr = re[j]
            let yw = w[j]
            let zr = re[l]
            let zw = w[l]
            // omega * (r, w) = (-w, r - w); omega^2 * (r, w) = (w - r, -r)
            re[i] = xr + yr + zr
            w[i] = xw + yw + zw
            re[j] = xr - yw + zw - zr
            w[j] = xw + yr - yw - zr
            re[l] = xr + yw - yr - zw
            w[l] = xw - yr + zr - zw
        base += 3 * stride
    stride *= 3

// real part of (r + w * omega)^3 = r^3 + w^3 - 3 r w^2
var cubes = 0
for c in 0..size:
    let r = re[c]
    let v = w[c]
    cubes += r * r * r + v * v * v - 3 * r * v * v
print((cubes / size - n) / 6)
