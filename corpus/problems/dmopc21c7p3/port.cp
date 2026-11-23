// Source: DMOPC '21 Contest 7 P3 (Whack-an-Ishigami), https://dmoj.ca/problem/dmopc21c7p3
// Problem: N <= 1e6 figures, some up (k_i = 1), and M <= 1e6 distinct links a -> b (a != b):
//   flipping a also flips b, in a chain reaction; an endless reaction resets the game. A whack
//   flips one figure. Print the fewest whacks that leave every figure down, or -1.
// Solution: a figure is flipped once per walk from a whacked figure to it, so on a DAG
//   flips(v) = whacked(v) + sum of flips over its in-links, and flips must equal k: v is
//   whacked iff k_v differs from the XOR of k over its in-links, a unique choice. Every whack
//   so chosen is a lit figure or reachable from one, and every lit figure needs a whack that
//   reaches it, so the answer is -1 iff a cycle is reachable from a lit figure (iterative DFS
//   with colours). O(N + M).
// original: corpus/submissions/4453037.cpp
let n: 1..=1000000 = read()
let m: 0..=1000000 = read()
domain V = 0..n
domain E = 0..m
let lit: [V]0..=1 = read()
let wires: [E](Int, Int) = read()

// forward adjacency as linked lists: head[u] -> edge, nxt[edge]
var head: [V]Int = -1
var nxt: [E]Int = -1
var after: [V]Int = 0
for e in E:
    let w = wires[e]
    let u = V(w.0 - 1)
    let v = V(w.1 - 1)
    nxt[e] = head[u]
    head[u] = e.to_idx
    after[v] = after[v] ^ lit[u]

// DFS colours over what the lit lamps reach: 1 = on the stack, 2 = done
var colour: [V]Int = 0
var edge: [V]Int = -1
var stack: List<Int> = []
var cyclic = false
for s in V:
    if lit[s] == 1 and colour[s] == 0:
        colour[s] = 1
        edge[s] = head[s]
        stack.push(s.to_idx)
        while stack.len > 0:
            let u = V(stack.last!)
            let e = edge[u]
            if e < 0:
                colour[u] = 2
                stack.pop()
            else:
                edge[u] = nxt[E(e)]
                let v = V(wires[E(e)].1 - 1)
                if colour[v] == 1:
                    cyclic = true
                elif colour[v] == 0:
                    colour[v] = 1
                    edge[v] = head[v]
                    stack.push(v.to_idx)

if cyclic:
    print(-1)
else:
    var ans = 0
    for v in V:
        ans += after[v] ^ lit[v]
    print(ans)
