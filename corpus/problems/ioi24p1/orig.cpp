// Not a judge-accepted submission: written for the corpus and checked against the sample and an exhaustive brute.
// Reads the sample grader's input (N, N lines "W A B", Q, Q lines E) and prints calculate_costs.
// Sorted by weight, a boat only ever pairs neighbours in runs whose adjacent gaps are <= D. A run of
// even length pays B for all; an odd run leaves one artifact alone, paying A - B for it: one at an
// even offset in the run, or any inner i with W[i+1] - W[i-1] <= D (its neighbours then pair over it).
// Taking D ascending, runs merge in a DSU that keeps, per run, the least A - B at each index parity
// and over the inner artifacts already skippable.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const ll INF = LLONG_MAX / 4;
vector<int> par, lo, len;
vector<array<ll, 2>> best;
vector<ll> skip;
ll extra = 0;
int find(int x) { return par[x] == x ? x : par[x] = find(par[x]); }
ll cost(int r) { return len[r] % 2 ? min(best[r][lo[r] % 2], skip[r]) : 0; }
int main() {
    cin.tie(nullptr)->sync_with_stdio(false);
    int n;
    cin >> n;
    vector<array<ll, 3>> a(n);
    for (auto& [w, x, y] : a) cin >> w >> x >> y;
    int q;
    cin >> q;
    vector<pair<ll, int>> e(q);
    for (int i = 0; i < q; i++) cin >> e[i].first, e[i].second = i;
    sort(a.begin(), a.end());
    par.resize(n), lo.resize(n), len.assign(n, 1), best.assign(n, {INF, INF}), skip.assign(n, INF);
    ll sumb = 0;
    for (int i = 0; i < n; i++) {
        par[i] = lo[i] = i;
        best[i][i % 2] = a[i][1] - a[i][2];
        sumb += a[i][2];
        extra += cost(i);
    }
    vector<tuple<ll, int, int>> ev;
    for (int i = 0; i + 1 < n; i++) ev.emplace_back(a[i + 1][0] - a[i][0], 0, i);
    for (int i = 1; i + 1 < n; i++) ev.emplace_back(a[i + 1][0] - a[i - 1][0], 1, i);
    sort(ev.begin(), ev.end());
    sort(e.begin(), e.end());
    vector<ll> res(q);
    size_t k = 0;
    for (auto [d, qi] : e) {
        for (; k < ev.size() && get<0>(ev[k]) <= d; k++) {
            auto [_, kind, i] = ev[k];
            if (kind == 0) {
                int x = find(i), y = find(i + 1);
                extra -= cost(x) + cost(y);
                par[y] = x;
                len[x] += len[y];
                for (int p = 0; p < 2; p++) best[x][p] = min(best[x][p], best[y][p]);
                skip[x] = min(skip[x], skip[y]);
                extra += cost(x);
            } else {
                int x = find(i);
                extra -= cost(x);
                skip[x] = min(skip[x], a[i][1] - a[i][2]);
                extra += cost(x);
            }
        }
        res[qi] = sumb + extra;
    }
    for (ll r : res) cout << r << "\n";
}
