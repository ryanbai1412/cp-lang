// Source: IOI 2012 P3 (Crayfish Scrivener), https://dmoj.ca/problem/ioi12p3
// Problem: a text starts empty. "T c" appends letter c, "U x" undoes the last x commands (an undo
//   is a command and can be undone too), "P i" prints the letter at 0-based position i (not a
//   command). Up to 1e6 operations; here stdin gives their count and then the lines.
// Solution: every text is an earlier text plus one letter, so the texts form a tree (parent =
//   the text without its last letter) and a command just names a node: T adds a child, U reuses
//   the node of an earlier command. "P i" climbs to the ancestor with i + 1 letters. Each node
//   keeps its parent and one skew-binary jump (its parent's jump's jump when the two jumps span
//   equal lengths, else its parent), so a climb is O(log n) steps and a new node is O(1).
// original: corpus/problems/ioi12p3/orig.cpp (submission 4682651 plus the sample grader)
let n: 1..=1e6 = read()
domain Text = 0..=n  // Text.first is the empty text
var len: [Text]Int = 0
var last: [Text]?Char = None
var up = [Text] t => t
var jump = [Text] t => t

var texts = 0  // texts made so far, not counting the empty one
var history: List<Text> = [Text.first!]
for _ in 0..n:
    let op: Char = read()
    if op == 'T':
        let letter: Char = read()
        let (old, now) = (history.last!, Text.nth(texts + 1))
        texts += 1
        len[now] = len[old] + 1
        last[now] = Some(letter)
        up[now] = old
        let j = jump[old]
        jump[now] = if len[old] - len[j] == len[j] - len[jump[j]]: jump[j] else: old
        history.push(now)
    elif op == 'U':
        let x: 1..=n = read()
        history.push(history[history.len - 1 - x])
    else:
        let i: 0..n = read()
        var v = history.last!
        while len[v] != i + 1:
            v = if len[jump[v]] > i: jump[v] else: up[v]
        print(last[v]!)
