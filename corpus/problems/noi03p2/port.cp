// Source: NOI 2003 P2 (Text Editor), https://dmoj.ca/problem/noi03p2
// limits: 250 ms, 64 MB
// Problem: a text, empty at first, and a cursor. t commands: "Move k", "Prev", "Next" place the
//   cursor; "Insert n" inserts the n characters (ASCII 32-126) that follow, line breaks
//   ignored; "Delete n" removes and "Get n" prints the n characters after the cursor. At most
//   4000 inserts and deletes, 2 MB inserted, 3 MB printed.
// Solution: with so few edits a piece table is enough: the text is a list of strings, cut
//   wherever an edit lands; an edit inserts or removes whole pieces in place (moving strings,
//   never copying them) and rebuilds the piece start offsets, O(pieces). Moving is free; a
//   Get binary-searches the offsets for the cursor's piece, then costs O(n) to print. Insert
//   text is read by lines, so spaces survive. The original uses a rope.
// Bounds: n: 0..=2_097_152
let t: Int = read()
var pieces: List<Str> = []
var starts = [0]  // starts[i]: where piece i begins; the last entry is the text's length
var cursor = 0

fn reindex():
    starts = [0]
    for s in pieces: starts.push(starts.last! + s.len)

// The piece holding position p; pieces.len when p is the end of the text.
fn piece_at(p: Int) -> Int = starts.bsearch(s => s > p) - 1

// Make position p a piece boundary; the number of pieces before it.
fn cut(p: Int) -> Int:
    let i = piece_at(p)
    if starts[i] == p: return i
    let (s, k) = (pieces[i].to_str(), p - starts[i])
    pieces[i] = s[0..k].to_str()
    pieces.insert(i + 1, s[k..s.len].to_str())
    reindex()
    i + 1

for _ in 0..t:
    let cmd: Str = read()
    if cmd == "Prev":
        cursor -= 1
    elif cmd == "Next":
        cursor += 1
    else:
        let n: 0..=2_097_152 = read()  // a position or a length in the 2 MB text
        if cmd == "Move":
            cursor = n
        elif cmd == "Insert":
            var text = read_line()
            while text.len < n: text += read_line()
            let i = cut(cursor)
            pieces.insert(i, text)
            reindex()
        elif cmd == "Delete":
            let (i, j) = (cut(cursor), cut(cursor + n))
            for _ in i..j: pieces.remove(i)
            reindex()
        else:
            var (shown, i) = ("", piece_at(cursor))
            while shown.len < n:
                let (s, at) = (pieces[i], starts[i])
                shown += s[max(cursor, at) - at..min(cursor + n, at + s.len) - at].to_str()
                i += 1
            print(shown)
