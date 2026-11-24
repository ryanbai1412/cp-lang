// ucrpc21i (Sphere Mongers) reference: the best disc of radius r can be moved until it
// either sits on a sphere or has two spheres on its rim, so the candidates are every sphere
// and the two centres through each pair at distance <= 2r. O(n^3) with n <= 1000.
#include <bits/stdc++.h>
using namespace std;

int main() {
    int n;
    long long r;
    cin >> n >> r;
    vector<double> x(n), y(n);
    vector<int> w(n);
    for (int i = 0; i < n; i++) {
        int g;
        cin >> x[i] >> y[i] >> g;
        w[i] = g ? 3 : 1;
    }
    const double rr = double(r) * r, eps = 1e-7;
    auto score = [&](double cx, double cy) {
        int s = 0;
        for (int i = 0; i < n; i++) {
            double dx = x[i] - cx, dy = y[i] - cy;
            if (dx * dx + dy * dy <= rr + eps) s += w[i];
        }
        return s;
    };
    int best = 0;
    for (int i = 0; i < n; i++) best = max(best, score(x[i], y[i]));
    for (int i = 0; i < n; i++)
        for (int j = i + 1; j < n; j++) {
            double dx = x[j] - x[i], dy = y[j] - y[i], d2 = dx * dx + dy * dy;
            if (d2 == 0 || d2 > 4 * rr) continue;
            double mx = (x[i] + x[j]) / 2, my = (y[i] + y[j]) / 2;
            double h = sqrt(max(0.0, rr - d2 / 4) / d2);
            best = max(best, score(mx - dy * h, my + dx * h));
            best = max(best, score(mx + dy * h, my - dx * h));
        }
    cout << best << "\n";
}
