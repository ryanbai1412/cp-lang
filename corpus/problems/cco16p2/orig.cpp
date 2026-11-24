// CCO 2016 P2 (Splitting Hares), written for the corpus: the accepted submission
// answers wrongly when points are collinear. For each pivot p, the line through p
// turns half a circle; at each direction its points are p and those q whose q - p
// is parallel. The line's two open sides plus any prefix or suffix of its points,
// in order along it, are every set a slight turn or shift of it takes.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
int main() {
    int n;
    scanf("%d", &n);
    vector<ll> x(n), y(n), w(n);
    ll total = 0;
    for (int i = 0; i < n; i++) scanf("%lld %lld %lld", &x[i], &y[i], &w[i]), total += w[i];
    ll best = LLONG_MIN;
    struct Q { ll dx, dy, w, dist; bool ahead; };
    vector<Q> qs;
    for (int p = 0; p < n; p++) {
        qs.clear();
        ll left = 0;
        for (int j = 0; j < n; j++) {
            if (j == p) continue;
            ll dx = x[j] - x[p], dy = y[j] - y[p];
            bool ahead = dy > 0 || (dy == 0 && dx > 0);
            if (!ahead) dx = -dx, dy = -dy;
            qs.push_back({dx, dy, w[j], llabs(dx) + llabs(dy), ahead});
            if (ahead) left += w[j];
        }
        sort(qs.begin(), qs.end(), [](const Q& a, const Q& b) {
            ll c = a.dx * b.dy - a.dy * b.dx;
            if (c != 0) return c > 0;
            return (a.ahead ? a.dist : -a.dist) < (b.ahead ? b.dist : -b.dist);
        });
        vector<ll> line;
        for (size_t i = 0; i < qs.size();) {
            size_t j = i;
            ll ahead = 0, behind = 0, on = w[p];
            while (j < qs.size() && qs[i].dx * qs[j].dy - qs[i].dy * qs[j].dx == 0) {
                (qs[j].ahead ? ahead : behind) += qs[j].w;
                on += qs[j].w;
                j++;
            }
            ll strict = left - ahead, other = total - strict - on;
            line.clear();
            size_t k = i;
            for (; k < j && !qs[k].ahead; k++) line.push_back(qs[k].w);
            line.push_back(w[p]);
            for (; k < j; k++) line.push_back(qs[k].w);
            for (ll side : {strict, other}) {
                ll s = 0;
                best = max(best, side);
                for (ll v : line) best = max(best, side + (s += v));
                s = 0;
                for (size_t t = line.size(); t-- > 0;) best = max(best, side + (s += line[t]));
            }
            left = strict + behind;
            i = j;
        }
    }
    printf("%lld\n", best);
}
