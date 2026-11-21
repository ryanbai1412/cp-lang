#include <bits/stdc++.h>
using namespace std;
using ll = long long;

struct Pt { ll x, y; };
struct Vec { double x, y; };

static ll area2(const vector<Pt>& poly) {
    int n = (int)poly.size();
    if (n < 3) return 0;
    ll total = 0;
    for (int i = 0; i < n; i++) {
        int j = (i + 1 == n) ? 0 : i + 1;
        total += poly[i].x * poly[j].y - poly[i].y * poly[j].x;
    }
    return total;
}

static double polygon_area(const vector<Vec>& poly) {
    int n = (int)poly.size();
    if (n < 3) return 0.0;
    double s = 0.0;
    for (int i = 0; i < n; i++) {
        int j = (i + 1 == n) ? 0 : i + 1;
        s += poly[i].x * poly[j].y - poly[i].y * poly[j].x;
    }
    return 0.5 * s;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    if (!(cin >> n)) return 0;
    vector<Pt> ipoly(n);
    vector<Vec> fpoly(n);
    for (int i = 0; i < n; i++) {
        cin >> ipoly[i].x >> ipoly[i].y;
        fpoly[i] = {(double)ipoly[i].x, (double)ipoly[i].y};
    }
    ll acc_i = 0;
    double acc_f = 0.0;
    for (int rep = 0; rep < 100; rep++) {
        acc_i += area2(ipoly);
        acc_f += polygon_area(fpoly);
    }
    cout << acc_i << ' ' << fixed << setprecision(2) << acc_f << '\n';
}
