#include <bits/stdc++.h>
using namespace std;

struct Vec {
    double x, y;
    Vec operator+(Vec o) const { return {x + o.x, y + o.y}; }
    Vec operator-(Vec o) const { return {x - o.x, y - o.y}; }
    Vec operator*(double k) const { return {x * k, y * k}; }
    Vec operator/(double k) const { return {x / k, y / k}; }
    double norm2() const { return x * x + y * y; }
    double len() const { return sqrt(norm2()); }
};

static double vcross(Vec a, Vec b) { return a.x * b.y - a.y * b.x; }

static bool circumcenter(Vec a, Vec b, Vec c, double eps, Vec &out) {
    Vec u = b - a, v = c - a;
    double den = 2.0 * vcross(u, v);
    if (fabs(den) <= eps) return false;
    double u2 = u.norm2(), v2 = v.norm2();
    out = a + Vec{v.y * u2 - u.y * v2, u.x * v2 - v.x * u2} / den;
    return true;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    cin >> n;
    vector<Vec> ps(n);
    for (int i = 0; i < n; i++) cin >> ps[i].x >> ps[i].y;
    long long rng = 20250101LL;
    for (int i = n - 1; i >= 1; i--) {
        rng = (rng * 1103515245LL + 12345LL) % 2147483647LL;
        int j = int(rng % (i + 1));
        swap(ps[i], ps[j]);
    }
    const double eps = 1e-9;
    Vec c = ps[0];
    double r = 0.0;
    for (int i = 0; i < n; i++) {
        if ((c - ps[i]).len() > r + eps) {
            c = ps[i];
            r = 0.0;
            for (int j = 0; j < i; j++) {
                if ((c - ps[j]).len() > r + eps) {
                    c = (ps[i] + ps[j]) * 0.5;
                    r = (c - ps[i]).len();
                    for (int k = 0; k < j; k++) {
                        if ((c - ps[k]).len() > r + eps) {
                            Vec o;
                            if (circumcenter(ps[i], ps[j], ps[k], eps, o)) {
                                c = o;
                                r = (c - ps[i]).len();
                            }
                        }
                    }
                }
            }
        }
    }
    cout << llround(c.x * 100.0) << ' ' << llround(c.y * 100.0) << ' ' << llround(r * 100.0) << '\n';
}
