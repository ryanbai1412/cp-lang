#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using i128 = __int128_t;

struct Pt {
    ll x, y;
    bool operator==(Pt o) const { return x == o.x && y == o.y; }
};

static inline i128 cross3(Pt a, Pt b, Pt c) {
    return (i128)(b.x - a.x) * (c.y - a.y) - (i128)(b.y - a.y) * (c.x - a.x);
}
static inline int turn(Pt a, Pt b, Pt c) {
    i128 v = cross3(a, b, c);
    return v > 0 ? 1 : (v < 0 ? -1 : 0);
}
static inline bool on_segment(Pt p, Pt a, Pt b) {
    if (cross3(p, a, b) != 0) return false;
    return (i128)(a.x - p.x) * (b.x - p.x) + (i128)(a.y - p.y) * (b.y - p.y) <= 0;
}

// 1 = Inside, 0 = Boundary, -1 = Outside
static int locate_convex(const vector<Pt>& hull, Pt p) {
    int n = (int)hull.size();
    if (n == 0) return -1;
    if (n == 1) return p == hull[0] ? 0 : -1;
    if (n == 2) return on_segment(p, hull[0], hull[1]) ? 0 : -1;
    int t1 = turn(hull[0], hull[1], p);
    int t2 = turn(hull[0], hull[n - 1], p);
    if (t1 < 0 || t2 > 0) return -1;
    if (t1 == 0) return on_segment(p, hull[0], hull[1]) ? 0 : -1;
    if (t2 == 0) return on_segment(p, hull[0], hull[n - 1]) ? 0 : -1;
    int lo = 1, hi = n - 1;
    while (hi - lo > 1) {
        int mid = (lo + hi) / 2;
        if (turn(hull[0], hull[mid], p) >= 0) lo = mid;
        else hi = mid;
    }
    int t = turn(hull[lo], hull[hi], p);
    return t > 0 ? 1 : (t == 0 ? 0 : -1);
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, q;
    if (!(cin >> n >> q)) return 0;
    vector<Pt> hull(n);
    for (int i = 0; i < n; i++) cin >> hull[i].x >> hull[i].y;
    ll ins = 0, bnd = 0, out = 0;
    for (int i = 0; i < q; i++) {
        Pt p;
        cin >> p.x >> p.y;
        int r = locate_convex(hull, p);
        if (r == 1) ins++;
        else if (r == 0) bnd++;
        else out++;
    }
    cout << ins << ' ' << bnd << ' ' << out << '\n';
}
