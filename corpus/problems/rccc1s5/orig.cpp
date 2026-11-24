// Reference for Riolku's Mock CCC S5 (no accepted submission on DMOJ; written for the corpus).
// Values stay below 2^18 (a_i, x <= 2e5). The multiset is a binary trie with counts and a
// lazy xor tag; `2 l r x` splits the values in [l, r] into their own trie, tags its root with
// x (swapping children wherever x has a bit) and merges it back. Merges free nodes, so the
// live tries never exceed 2^19 nodes; each split adds O(18) and merges are amortised by them.
#include <bits/stdc++.h>
using namespace std;

const int B = 18, TOP = (1 << B) - 1;
struct Node { int ch[2], cnt, xr, tag; };
vector<Node> t(1);
vector<int> freed;

int node() {
    if (!freed.empty()) { int u = freed.back(); freed.pop_back(); t[u] = Node{}; return u; }
    t.push_back(Node{});
    return (int)t.size() - 1;
}
void drop(int u) { freed.push_back(u); }

void apply(int u, int d, int x) {
    if (!u) return;
    if (d < B && (x >> (B - 1 - d) & 1)) swap(t[u].ch[0], t[u].ch[1]);
    if (t[u].cnt & 1) t[u].xr ^= x;
    t[u].tag ^= x;
}
void push(int u, int d) {
    if (d < B && t[u].tag) {
        apply(t[u].ch[0], d + 1, t[u].tag);
        apply(t[u].ch[1], d + 1, t[u].tag);
    }
    t[u].tag = 0;
}
int pull(int u) {
    int a = t[u].ch[0], b = t[u].ch[1];
    t[u].cnt = t[a].cnt + t[b].cnt;
    t[u].xr = t[a].xr ^ t[b].xr;
    if (!t[u].cnt) { drop(u); return 0; }
    return u;
}

int insert(int u, int d, int v) {
    if (!u) u = node();
    if (d == B) { t[u].cnt++; t[u].xr ^= v; return u; }
    push(u, d);
    int b = v >> (B - 1 - d) & 1;
    int c = insert(t[u].ch[b], d + 1, v);
    t[u].ch[b] = c;
    return pull(u);
}

// Splits u (covering [lo, hi]) into (values outside [l, r], values inside).
pair<int, int> split(int u, int d, int lo, int hi, int l, int r) {
    if (!u || r < lo || hi < l) return {u, 0};
    if (l <= lo && hi <= r) return {0, u};
    push(u, d);
    int mid = (lo + hi) / 2;
    auto [a0, b0] = split(t[u].ch[0], d + 1, lo, mid, l, r);
    auto [a1, b1] = split(t[u].ch[1], d + 1, mid + 1, hi, l, r);
    int v = node();
    t[v].ch[0] = b0, t[v].ch[1] = b1;
    t[u].ch[0] = a0, t[u].ch[1] = a1;
    return {pull(u), pull(v)};
}

int merge(int u, int v, int d) {
    if (!u || !v) return u ^ v;
    if (d == B) { t[u].cnt += t[v].cnt; t[u].xr ^= t[v].xr; drop(v); return u; }
    push(u, d), push(v, d);
    int c0 = merge(t[u].ch[0], t[v].ch[0], d + 1);
    int c1 = merge(t[u].ch[1], t[v].ch[1], d + 1);
    t[u].ch[0] = c0, t[u].ch[1] = c1;
    drop(v);
    return pull(u);
}

int query(int u, int d, int lo, int hi, int l, int r) {
    if (!u || r < lo || hi < l) return 0;
    if (l <= lo && hi <= r) return t[u].xr;
    push(u, d);
    int mid = (lo + hi) / 2;
    return query(t[u].ch[0], d + 1, lo, mid, l, r) ^ query(t[u].ch[1], d + 1, mid + 1, hi, l, r);
}

int main() {
    int n, q;
    scanf("%d %d", &n, &q);
    int root = 0;
    for (int i = 0; i < n; i++) { int a; scanf("%d", &a); root = insert(root, 0, a); }
    string out;
    while (q--) {
        int ty;
        scanf("%d", &ty);
        if (ty == 1) {
            int x; scanf("%d", &x);
            root = insert(root, 0, x);
        } else if (ty == 2) {
            int l, r, x; scanf("%d %d %d", &l, &r, &x);
            if (l > TOP) continue;
            auto [rest, ext] = split(root, 0, 0, TOP, l, min(r, TOP));
            apply(ext, 0, x);
            root = merge(rest, ext, 0);
        } else {
            int l, r; scanf("%d %d", &l, &r);
            out += to_string(l > TOP ? 0 : query(root, 0, 0, TOP, l, min(r, TOP))) + "\n";
        }
    }
    fputs(out.c_str(), stdout);
}
