#include <bits/stdc++.h>
using namespace std;

struct Vec {
    double x, y;
    Vec operator+(Vec o) const { return {x + o.x, y + o.y}; }
    Vec operator-(Vec o) const { return {x - o.x, y - o.y}; }
    Vec operator*(double k) const { return {x * k, y * k}; }
    Vec perp() const { return {-y, x}; }
    double len() const { return sqrt(x * x + y * y); }
};

static inline double vcross(Vec a, Vec b) { return a.x * b.y - a.y * b.x; }

struct Iv {
    double lo;
    Vec p_lo;
    double hi;
    Vec p_hi;
};

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int raw_n;
    cin >> raw_n;
    vector<pair<Vec, double>> circles(raw_n);
    for (int i = 0; i < raw_n; i++) cin >> circles[i].first.x >> circles[i].first.y >> circles[i].second;
    const double eps = 1e-9;
    const double pi = 3.14159265358979323846;
    vector<pair<Vec, double>> cs;
    for (int i = 0; i < raw_n; i++) {
        if (circles[i].second <= eps) continue;
        bool covered = false;
        for (int j = 0; j < raw_n; j++) {
            if (i == j || circles[j].second <= eps) continue;
            double d = (circles[i].first - circles[j].first).len();
            if (d + circles[i].second <= circles[j].second + eps) {
                bool same = d <= eps && fabs(circles[i].second - circles[j].second) <= eps;
                if (!same || j < i) {
                    covered = true;
                    break;
                }
            }
        }
        if (!covered) cs.push_back(circles[i]);
    }
    int n = (int)cs.size();
    double total = 0.0;
    vector<Iv> ivs;
    for (int i = 0; i < n; i++) {
        Vec ci = cs[i].first;
        double ri = cs[i].second;
        Vec seam{ci.x - ri, ci.y};
        ivs.clear();
        for (int j = 0; j < n; j++) {
            if (i == j) continue;
            Vec dv = cs[j].first - ci;
            double d = dv.len();
            if (d >= ri + cs[j].second - eps || d <= eps) continue;
            double x = (d * d + ri * ri - cs[j].second * cs[j].second) / (2.0 * d);
            double h = sqrt(max(0.0, ri * ri - x * x));
            if (h <= eps) continue;
            Vec base = ci + dv * (x / d);
            Vec off = dv.perp() * (h / d);
            Vec p1 = base - off, p2 = base + off;
            double a1 = atan2(p1.y - ci.y, p1.x - ci.x);
            double a2 = atan2(p2.y - ci.y, p2.x - ci.x);
            if (a1 <= a2) {
                ivs.push_back({a1, p1, a2, p2});
            } else {
                ivs.push_back({a1, p1, pi, seam});
                ivs.push_back({-pi, seam, a2, p2});
            }
        }
        if (ivs.empty()) {
            total += pi * ri * ri;
            continue;
        }
        sort(ivs.begin(), ivs.end(), [](const Iv &a, const Iv &b) { return a.lo < b.lo; });
        double cur_ang = -pi;
        Vec cur_pt = seam;
        for (const auto &iv : ivs) {
            if (iv.lo > cur_ang) {
                total += 0.5 * (ri * ri * (iv.lo - cur_ang) + vcross(ci, iv.p_lo - cur_pt));
            }
            if (iv.hi > cur_ang) {
                cur_ang = iv.hi;
                cur_pt = iv.p_hi;
            }
        }
        if (pi > cur_ang) {
            total += 0.5 * (ri * ri * (pi - cur_ang) + vcross(ci, seam - cur_pt));
        }
    }
    cout << llround(total * 1000.0) << '\n';
}
