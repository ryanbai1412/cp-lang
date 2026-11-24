// NOI 2010 Nemo (output-only; scored against thresholds on the eaten weight). Greedy
// chases: from where and when Nemo is, every shrimp lighter than him has an earliest
// interception time (the least root of |s(t) - pos| = V (t - now)); the next target is the
// one with the earliest interception, or the most weight per unit of time spent. The heavier
// plan wins. checker.py accepts any valid plan that eats at least as much.
#include <bits/stdc++.h>
using namespace std;
using ld = long double;

int n;
ld W0, V, T, X0, Y0;
vector<ld> w, x, y, p, q;

ld intercept(int i, ld t, ld px, ld py) {
    ld dx = x[i] + p[i] * t - px, dy = y[i] + q[i] * t - py;
    ld a = p[i] * p[i] + q[i] * q[i] - V * V, b = 2 * (dx * p[i] + dy * q[i]), c = dx * dx + dy * dy;
    if (c < 1e-18) return 0;
    if (fabsl(a) < 1e-12) return b < 0 ? -c / b : INFINITY;
    ld disc = b * b - 4 * a * c;
    if (disc < 0) return INFINITY;
    ld s = sqrtl(disc), r1 = (-b - s) / (2 * a), r2 = (-b + s) / (2 * a);
    if (r1 > r2) swap(r1, r2);
    if (r1 >= 0) return r1;
    return r2 >= 0 ? r2 : INFINITY;
}

struct Plan { ld total = 0; vector<tuple<ld, ld, ld, int>> rows; };

Plan chase(int rule) {
    Plan plan;
    vector<char> eaten(n);
    ld t = 0, px = X0, py = Y0, cw = W0;
    while (true) {
        int best = -1;
        ld bt = 0, bkey = 0;
        for (int i = 0; i < n; i++) {
            if (eaten[i] || w[i] >= cw) continue;
            ld d = intercept(i, t, px, py);
            if (!isfinite(d)) continue;
            ld at = t + d * (1 + 1e-12) + 1e-9;
            if (at > T) continue;
            ld key = rule == 0 ? -at : w[i] / (at - t + (rule == 1 ? 1e-3 : 1.0));
            if (best < 0 || key > bkey) best = i, bt = at, bkey = key;
        }
        if (best < 0) break;
        eaten[best] = 1, t = bt, px = x[best] + p[best] * t, py = y[best] + q[best] * t, cw += w[best];
        plan.total += w[best], plan.rows.emplace_back(t, px, py, best + 1);
    }
    return plan;
}

int main() {
    int tc;
    cin >> tc >> W0 >> V >> T >> X0 >> Y0 >> n;
    w.resize(n), x.resize(n), y.resize(n), p.resize(n), q.resize(n);
    for (int i = 0; i < n; i++) cin >> w[i] >> x[i] >> y[i] >> p[i] >> q[i];
    Plan best;
    for (int rule = 0; rule < 3; rule++) {
        Plan pl = chase(rule);
        if (rule == 0 || pl.total > best.total) best = pl;
    }
    printf("%zu\n%.10Lf\n", best.rows.size(), best.total);
    for (auto& [t, px, py, s] : best.rows) printf("%.10Lf %.10Lf %.10Lf %d\n", t, px, py, s);
}
