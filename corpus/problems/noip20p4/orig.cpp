// Reference for NOIP 2020 P4 (no accepted C++ on DMOJ: its submissions are RTE or WA; written
// for the corpus). The answer is the sum over t >= 0 of the starts still inside after t steps.
// Per dimension j the inside starts after t steps are w_j minus the width the walk has swept.
// The first round is summed step by step. From the second round on, after k >= 1 rounds and i
// more steps the swept width is D_j + a_j(i) + (k - 1)|e_j| (D_j: the first round's width,
// a_j(i): what the second round's first i steps add, e_j: one round's drift), so for each i
// the sum over k is a sum of a degree-<=10 polynomial in k, done with power-sum polynomials.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
const ll MOD = 1e9 + 7;

ll pw(ll b, ll e) {
    ll r = 1;
    b %= MOD;
    for (; e; e >>= 1, b = b * b % MOD)
        if (e & 1) r = r * b % MOD;
    return r;
}

int main() {
    int n, k;
    scanf("%d %d", &n, &k);
    vector<ll> w(k);
    for (auto &x : w) scanf("%lld", &x);
    vector<int> c(n), d(n);
    for (int i = 0; i < n; i++) scanf("%d %d", &c[i], &d[i]), c[i]--;
    vector<ll> pos(k, 0), lo(k, 0), hi(k, 0);
    ll ans = 0;
    auto alive = [&]() {
        ll prod = 1;
        for (int j = 0; j < k; j++) {
            ll a = w[j] - (hi[j] - lo[j]);
            if (a <= 0) return 0LL;
            prod = prod * (a % MOD) % MOD;
        }
        return prod;
    };
    for (int i = 0; i < n; i++) {
        ll a = alive();
        if (!a) { printf("%lld\n", ans); return 0; }
        ans = (ans + a) % MOD;
        int j = c[i];
        pos[j] += d[i], lo[j] = min(lo[j], pos[j]), hi[j] = max(hi[j], pos[j]);
    }
    vector<ll> D(k), e(pos);
    for (int j = 0; j < k; j++) D[j] = hi[j] - lo[j];
    bool still = alive() != 0, drift = false;
    for (int j = 0; j < k; j++) drift |= e[j] != 0;
    if (!still) { printf("%lld\n", ans); return 0; }
    if (!drift) { puts("-1"); return 0; }
    // Power sums S_p(X) = sum_{x=0}^{X} x^p as polynomials in X (degree p + 1), p <= k.
    vector<vector<ll>> C(k + 2, vector<ll>(k + 2, 0));
    for (int i = 0; i <= k + 1; i++) {
        C[i][0] = 1;
        for (int j = 1; j <= i; j++) C[i][j] = (C[i - 1][j - 1] + C[i - 1][j]) % MOD;
    }
    vector<vector<ll>> S(k + 1, vector<ll>(k + 2, 0));
    for (int p = 0; p <= k; p++) {
        // (p + 1) S_p = (X + 1)^{p + 1} - sum_{q < p} C(p + 1, q) S_q
        vector<ll> f(k + 2, 0);
        for (int q = 0; q <= p + 1; q++) f[q] = C[p + 1][q];
        for (int q = 0; q < p; q++)
            for (int t = 0; t <= k + 1; t++) f[t] = (f[t] - C[p + 1][q] * S[q][t] % MOD + MOD) % MOD;
        ll inv = pw(p + 1, MOD - 2);
        for (int t = 0; t <= k + 1; t++) S[p][t] = f[t] * inv % MOD;
    }
    // Second round, step by step.
    vector<ll> poly(k + 1);
    for (int i = 0; i < n; i++) {
        // after i steps of round 2: width_j = (hi - lo) now; b_j = w_j - width_j, slope |e_j|.
        ll X = LLONG_MAX;
        bool dead = false;
        for (int j = 0; j < k; j++) {
            ll b = w[j] - (hi[j] - lo[j]);
            if (b <= 0) { dead = true; break; }
            if (e[j]) X = min(X, (b - 1) / llabs(e[j]));
        }
        if (dead) break;
        // prod_j (b_j - |e_j| x), x = 0..X
        fill(poly.begin(), poly.end(), 0);
        poly[0] = 1;
        int deg = 0;
        for (int j = 0; j < k; j++) {
            ll b = (w[j] - (hi[j] - lo[j])) % MOD, s = (MOD - llabs(e[j]) % MOD) % MOD;
            for (int t = deg + 1; t >= 0; t--) {
                ll v = poly[t] * b % MOD;
                if (t) v = (v + poly[t - 1] * s) % MOD;
                poly[t] = v;
            }
            deg++;
        }
        ll Xm = X % MOD, sum = 0;
        for (int p = 0; p <= k; p++) {
            if (!poly[p]) continue;
            ll val = 0, xp = 1;
            for (int t = 0; t <= p + 1; t++, xp = xp * Xm % MOD) val = (val + S[p][t] * xp) % MOD;
            sum = (sum + poly[p] * val) % MOD;
        }
        ans = (ans + sum) % MOD;
        int j = c[i];
        pos[j] += d[i], lo[j] = min(lo[j], pos[j]), hi[j] = max(hi[j], pos[j]);
    }
    printf("%lld\n", ans);
}
