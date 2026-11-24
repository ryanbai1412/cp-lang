// sdoi17r2p5 (SDOI 2017 Forgotten Set) reference. With a_d = [d in S],
// F = prod_{d in S} 1/(1 - x^d), so x F'/F = sum_k x^k sum_{d | k} d a_d. Each d < p is
// invertible mod p, so S is the unique set fitting f, read off by a divisor sieve over
// b = x F'/F. F'/F is a power-series inverse mod an arbitrary p: Newton iteration with
// three-prime NTT products recombined by CRT.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
typedef unsigned long long ull;
typedef __int128 L;

ll P;
ll pw(ll b, ll e, ll m) { ll r = 1; b %= m; for (; e; e >>= 1, b = (L)b * b % m) if (e & 1) r = (L)r * b % m; return r; }

template <ll M>
void ntt(vector<ll>& a, bool inv) {
    int n = a.size();
    for (int i = 1, j = 0; i < n; i++) {
        int bit = n >> 1;
        for (; j & bit; bit >>= 1) j ^= bit;
        j ^= bit;
        if (i < j) swap(a[i], a[j]);
    }
    for (int len = 2; len <= n; len <<= 1) {
        ll w = pw(3, (M - 1) / len, M);
        if (inv) w = pw(w, M - 2, M);
        for (int i = 0; i < n; i += len) {
            ll wn = 1;
            for (int j = 0; j < len / 2; j++) {
                ll u = a[i + j], v = a[i + j + len / 2] * wn % M;
                a[i + j] = u + v < M ? u + v : u + v - M;
                a[i + j + len / 2] = u - v >= 0 ? u - v : u - v + M;
                wn = wn * w % M;
            }
        }
    }
    if (inv) { ll ni = pw(n, M - 2, M); for (auto& x : a) x = x * ni % M; }
}

template <ll M>
vector<ll> convMod(const vector<ll>& a, const vector<ll>& b, int sz) {
    vector<ll> x(sz, 0), y(sz, 0);
    for (size_t i = 0; i < a.size(); i++) x[i] = a[i] % M;
    for (size_t i = 0; i < b.size(); i++) y[i] = b[i] % M;
    ntt<M>(x, false), ntt<M>(y, false);
    for (int i = 0; i < sz; i++) x[i] = x[i] * y[i] % M;
    ntt<M>(x, true);
    return x;
}

// a * b mod P, first `keep` coefficients
vector<ll> mul(const vector<ll>& a, const vector<ll>& b, int keep) {
    const ll M1 = 998244353, M2 = 167772161, M3 = 469762049;
    int need = a.size() + b.size() - 1, sz = 1;
    while (sz < need) sz <<= 1;
    auto r1 = convMod<M1>(a, b, sz), r2 = convMod<M2>(a, b, sz), r3 = convMod<M3>(a, b, sz);
    const ll i12 = pw(M1, M2 - 2, M2), i123 = pw((L)M1 * M2 % M3, M3 - 2, M3);
    vector<ll> r(keep, 0);
    for (int i = 0; i < keep && i < need; i++) {
        ll x1 = r1[i];
        ll x2 = (r2[i] - x1 % M2 + M2) % M2 * i12 % M2;  // value = x1 + M1 x2 + M1 M2 x3
        ll v12 = (x1 + (L)M1 * x2) % M3;
        ll x3 = (r3[i] - v12 + M3) % M3 * i123 % M3;
        r[i] = (ll)(((L)x1 + (L)M1 * x2 + (L)M1 * M2 % P * x3) % P);
    }
    return r;
}

vector<ll> inverse(const vector<ll>& f, int n) {
    vector<ll> g = {pw(f[0], P - 2, P)};
    for (int len = 1; len < n;) {
        len = min(2 * len, n);
        vector<ll> fl(f.begin(), f.begin() + min<size_t>(len, f.size()));
        vector<ll> t = mul(fl, g, len);  // f g
        for (auto& x : t) x = (P - x) % P;
        t[0] = (t[0] + 2) % P;  // 2 - f g
        g = mul(g, t, len);
    }
    return g;
}

int main() {
    int n;
    scanf("%d %lld", &n, &P);
    vector<ll> F(n + 1);
    F[0] = 1;
    for (int i = 1; i <= n; i++) scanf("%lld", &F[i]);
    vector<ll> dF(n);
    for (int i = 1; i <= n; i++) dF[i - 1] = F[i] * i % P;
    vector<ll> q = mul(dF, inverse(F, n), n);  // F'/F, coefficient k-1 is b_k
    vector<ll> b(n + 1, 0);
    for (int k = 1; k <= n; k++) b[k] = q[k - 1];
    vector<int> s;
    for (int d = 1; d <= n; d++) {
        if (b[d] == 0) continue;
        if (b[d] != d % P) { fprintf(stderr, "no set fits f at %d\n", d); return 1; }
        s.push_back(d);
        for (int k = 2 * d; k <= n; k += d) b[k] = (b[k] - d % P + P) % P;
    }
    printf("%zu\n", s.size());
    for (size_t i = 0; i < s.size(); i++) printf("%d%c", s[i], i + 1 == s.size() ? '\n' : ' ');
}
