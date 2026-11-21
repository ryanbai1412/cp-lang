#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using i128 = __int128_t;

struct Pt {
    ll x, y;
    Pt operator+(Pt o) const { return {x + o.x, y + o.y}; }
    Pt operator-(Pt o) const { return {x - o.x, y - o.y}; }
    bool operator!=(Pt o) const { return x != o.x || y != o.y; }
    bool operator==(Pt o) const { return x == o.x && y == o.y; }
};

static inline i128 cross128(Pt a, Pt b) { return (i128)a.x * b.y - (i128)a.y * b.x; }

static int lowest_leftmost(const vector<Pt>& poly) {
    int best = 0;
    for (int i = 1; i < (int)poly.size(); i++) {
        if (poly[i].y < poly[best].y || (poly[i].y == poly[best].y && poly[i].x < poly[best].x))
            best = i;
    }
    return best;
}

static vector<Pt> minkowski_sum(const vector<Pt>& a, const vector<Pt>& b) {
    if (a.empty() || b.empty()) return {};
    int na = (int)a.size(), nb = (int)b.size();
    int sa = lowest_leftmost(a), sb = lowest_leftmost(b);
    vector<Pt> res;
    res.reserve(na + nb);
    int i = 0, j = 0;
    while (i < na || j < nb) {
        Pt cur = a[(sa + i) % na] + b[(sb + j) % nb];
        if (res.empty() || res.back() != cur) res.push_back(cur);
        if (i == na) {
            j++;
        } else if (j == nb) {
            i++;
        } else {
            Pt ea = a[(sa + i + 1) % na] - a[(sa + i) % na];
            Pt eb = b[(sb + j + 1) % nb] - b[(sb + j) % nb];
            i128 c = cross128(ea, eb);
            if (c > 0) i++;
            else if (c < 0) j++;
            else { i++; j++; }
        }
    }
    while (res.size() > 1 && res.back() == res[0]) res.pop_back();
    return res;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int na, nb;
    if (!(cin >> na >> nb)) return 0;
    vector<Pt> a(na), b(nb);
    for (int i = 0; i < na; i++) cin >> a[i].x >> a[i].y;
    for (int i = 0; i < nb; i++) cin >> b[i].x >> b[i].y;
    ll checksum = 0;
    size_t last_len = 0;
    for (int rep = 0; rep < 20; rep++) {
        auto s = minkowski_sum(a, b);
        last_len = s.size();
        for (const auto& p : s) checksum += p.x + p.y;
    }
    cout << last_len << ' ' << checksum << '\n';
}
