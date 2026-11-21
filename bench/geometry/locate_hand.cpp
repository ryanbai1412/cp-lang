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
static inline bool on_segment(Pt p, Pt a, Pt b) {
    if (cross3(p, a, b) != 0) return false;
    return (i128)(a.x - p.x) * (b.x - p.x) + (i128)(a.y - p.y) * (b.y - p.y) <= 0;
}

// 1 = Inside, 0 = Boundary, -1 = Outside
static int locate(const vector<Pt>& poly, Pt p) {
    int n = (int)poly.size();
    if (n == 0) return -1;
    if (n == 1) return p == poly[0] ? 0 : -1;
    bool inside = false;
    Pt prev = poly[n - 1];
    for (int i = 0; i < n; i++) {
        Pt cur = poly[i];
        Pt a = prev, b = cur;
        prev = cur;
        if (a.y > b.y) swap(a, b);
        if (a.y <= p.y && p.y <= b.y) {
            i128 c = cross3(p, a, b);
            if (c == 0) {
                if ((i128)(a.x - p.x) * (b.x - p.x) + (i128)(a.y - p.y) * (b.y - p.y) <= 0) return 0;
            } else if (p.y < b.y && c > 0) {
                inside = !inside;
            }
        }
    }
    return inside ? 1 : -1;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, q;
    if (!(cin >> n >> q)) return 0;
    vector<Pt> poly(n);
    for (int i = 0; i < n; i++) cin >> poly[i].x >> poly[i].y;
    ll ins = 0, bnd = 0, out = 0;
    for (int i = 0; i < q; i++) {
        Pt p;
        cin >> p.x >> p.y;
        int r = locate(poly, p);
        if (r == 1) ins++;
        else if (r == 0) bnd++;
        else out++;
    }
    cout << ins << ' ' << bnd << ' ' << out << '\n';
}
