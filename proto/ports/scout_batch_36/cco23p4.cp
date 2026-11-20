// CCO '23 P4 — Binary String
// original: corpus/submissions/5579929.cpp
//
// Minimum swaps to remove every occurrence of a forbidden binary pattern t
// (|t| <= 3) from s. Complement symmetry halves the 14 patterns to 7 canonical
// cases; 000 reduces to greedily breaking runs of >= 3 zeros using 0-runs of
// length 0 or 1 between adjacent ones.

let s: Str = read()
let t: Str = read()

fn flip(w: Str) -> Str:
    var out = ""
    for ch in w: out.push(if ch == '0': '1' else: '0')
    out

fn count_sub(w: Str, pat: Str) -> Int:
    var res = 0
    for i in 0..=w.len - pat.len:
        if w[i..i + pat.len] == pat: res += 1
    res

fn solve0(w: Str) -> Int:
    if count_sub(w, "0") > 0: -1 else: 0

fn solve00(w: Str) -> Int:
    let c0 = count_sub(w, "0")
    if c0 > w.len - c0 + 1: -1 else: count_sub(w, "00")

fn solve000(w: Str) -> Int:
    let c0 = count_sub(w, "0")
    if c0 > 2 * (w.len - c0 + 1): return -1
    var gaps: List<Int> = []
    var run = 0
    for ch in w:
        if ch == '1':
            gaps.push(run)
            run = 0
        else:
            run += 1
    gaps.push(run)

    var cnt0 = 0
    var cnt1 = 0
    for g in gaps:
        if g == 0: cnt0 += 1
        elif g == 1: cnt1 += 1

    var ans = 0
    for g0 in gaps:
        var g = g0
        while g > 2:
            if g == 3:
                if cnt1 > 0: cnt1 -= 1
                else:
                    cnt0 -= 1
                    cnt1 += 1
                g -= 1
            else:
                if cnt0 > 0:
                    cnt0 -= 1
                    g -= 2
                else:
                    cnt1 -= 1
                    g -= 1
            ans += 1
    ans

let cs = flip(s)
let ans = match t:
    "0" => solve0(s)
    "1" => solve0(cs)
    "00" => solve00(s)
    "11" => solve00(cs)
    "01" => count_sub(s, "01")
    "10" => count_sub(cs, "01")
    "010" => (count_sub(s, "010") + 1) / 2
    "101" => (count_sub(cs, "010") + 1) / 2
    "001" => count_sub(s, "001")
    "110" => count_sub(cs, "001")
    "011" => count_sub(s, "011")
    "100" => count_sub(cs, "011")
    "000" => solve000(s)
    _ => solve000(cs)
print(ans)
