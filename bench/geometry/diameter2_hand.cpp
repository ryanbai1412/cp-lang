#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using i128 = __int128_t;

struct Pt {
    ll x, y;
    Pt operator-(Pt o) const { return {x - o.x, y - o.y}; }
};

static inline i128 cross128(Pt a, Pt b) { return (i128)a.x * b.y - (i128)a.y * b.x; }
static inline ll dist2(Pt a, Pt b) { ll dx = a.x - b.x, dy = a.y - b.y; return dx * dx + dy * dy; }

static ll diameter2(const vector<Pt>& hull) {
    int n = (int)hull.size();
    if (n <= 1) return 0;
    if (n == 2) return dist2(hull[0], hull[1]);
    ll best = 0;
    int j = 1;
    for (int i = 0; i < n; i++) {
        int ni = (i + 1) % n;
        Pt e = hull[ni] - hull[i];
        while (true) {
            int nj = (j + 1) % n;
            if (cross128(e, hull[nj] - hull[i]) > cross128(e, hull[j] - hull[i])) {
                j = nj;
            } else {
                break;
            }
        }
        best = max(best, dist2(hull[i], hull[j]));
        best = max(best, dist2(hull[ni], hull[j]));
    }
    return best;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Pt> hull(n);
    for (int i = 0; i < n; i++) cin >> hull[i].x >> hull[i].y;
    ll ans = 0;
    for (int rep = 0; rep < 50; rep++) ans = diameter2(hull);
    cout << ans << '\n';
}
