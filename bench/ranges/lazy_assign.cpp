#include <algorithm>
#include <iostream>
#include <vector>
using namespace std;
using ll = long long;

struct Node {
    int holes, cells, best, pre, suf;
    Node operator+(const Node& b) const {
        return {holes + b.holes, cells + b.cells, max({best, b.best, suf + b.pre}),
                cells == 0 ? holes + b.pre : pre, b.cells == 0 ? b.holes + suf : b.suf};
    }
    static Node solid(int hole, int len) {
        return hole ? Node{len, 0, len, len, len} : Node{0, len, 0, 0, 0};
    }
};

struct ST {
    int n;
    vector<Node> v;
    vector<int> lazy;
    ST(int n) : n(n), v(4 * n), lazy(4 * n, -1) { build(1, 0, n - 1); }
    void build(int o, int l, int r) {
        if (l == r) { v[o] = Node::solid(0, 1); return; }
        int mid = (l + r) / 2;
        build(2 * o, l, mid);
        build(2 * o + 1, mid + 1, r);
        v[o] = v[2 * o] + v[2 * o + 1];
    }
    void maintain(int o, int l, int r) {
        if (lazy[o] != -1) v[o] = Node::solid(lazy[o], r - l + 1);
        else if (l < r) v[o] = v[2 * o] + v[2 * o + 1];
    }
    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) { lazy[o] = x; maintain(o, l, r); return; }
        int mid = (l + r) / 2, lc = 2 * o, rc = lc + 1;
        if (lazy[o] != -1) { lazy[lc] = lazy[rc] = lazy[o]; lazy[o] = -1; }
        if (ql <= mid) update(lc, l, mid, ql, qr, x); else maintain(lc, l, mid);
        if (mid < qr) update(rc, mid + 1, r, ql, qr, x); else maintain(rc, mid + 1, r);
        maintain(o, l, r);
    }
    Node query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) return v[o];
        if (lazy[o] != -1) return Node::solid(lazy[o], min(r, qr) - max(l, ql) + 1);
        int mid = (l + r) / 2;
        if (ql <= mid) {
            if (mid < qr) return query(2 * o, l, mid, ql, qr) + query(2 * o + 1, mid + 1, r, ql, qr);
            return query(2 * o, l, mid, ql, qr);
        }
        return query(2 * o + 1, mid + 1, r, ql, qr);
    }
};

int main() {
    const int n = 200000, q = 200000;
    ll seed = 123456789;
    auto next_rng = [&] { seed = (seed * 1103515245 + 12345) % 2147483648LL; return seed; };
    ST st(n);
    ll acc = 0;
    for (int step = 0; step < q; step++) {
        int a = next_rng() % n, b = next_rng() % n;
        int lo = min(a, b), hi = max(a, b);
        int op = step % 3;
        if (op == 0) st.update(1, 0, n - 1, lo, hi, 1);
        else if (op == 1) st.update(1, 0, n - 1, lo, hi, 0);
        else acc += st.query(1, 0, n - 1, lo, hi).best;
    }
    cout << acc << "\n";
}
