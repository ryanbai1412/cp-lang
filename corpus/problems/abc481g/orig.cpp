// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const ll MOD = 998244353;
ll pw(ll b, ll e) { ll r = 1; b %= MOD; for (; e; e >>= 1, b = b * b % MOD) if (e & 1) r = r * b % MOD; return r; }
void ntt(vector<ll> &a, bool inv) {
    int n = a.size();
    for (int i = 1, j = 0; i < n; i++) {
        int bit = n >> 1;
        for (; j & bit; bit >>= 1) j ^= bit;
        j ^= bit;
        if (i < j) swap(a[i], a[j]);
    }
    for (int len = 2; len <= n; len <<= 1) {
        ll w = pw(3, (MOD - 1) / len);
        if (inv) w = pw(w, MOD - 2);
        for (int i = 0; i < n; i += len) {
            ll wn = 1;
            for (int j = 0; j < len / 2; j++, wn = wn * w % MOD) {
                ll u = a[i + j], v = a[i + j + len / 2] * wn % MOD;
                a[i + j] = (u + v) % MOD, a[i + j + len / 2] = (u - v + MOD) % MOD;
            }
        }
    }
    if (inv) { ll ni = pw(n, MOD - 2); for (auto &x : a) x = x * ni % MOD; }
}
vector<ll> mul(vector<ll> a, vector<ll> b) {
    int need = a.size() + b.size() - 1, sz = 1;
    if (min(a.size(), b.size()) <= 32) {
        vector<ll> c(need, 0);
        for (size_t i = 0; i < a.size(); i++) for (size_t j = 0; j < b.size(); j++) c[i + j] = (c[i + j] + a[i] * b[j]) % MOD;
        return c;
    }
    while (sz < need) sz <<= 1;
    a.resize(sz), b.resize(sz), ntt(a, false), ntt(b, false);
    for (int i = 0; i < sz; i++) a[i] = a[i] * b[i] % MOD;
    ntt(a, true), a.resize(need);
    return a;
}
int n;
vector<ll> fact, ifact;
vector<ll> binom(int d) { vector<ll> c(d + 1); for (int i = 0; i <= d; i++) c[i] = fact[d] * ifact[i] % MOD * ifact[d - i] % MOD; return c; }
struct Gap { int p, q; ll w; };
vector<Gap> gaps;
struct Seg { vector<ll> poly; int p, q; };
Seg merge(int l, int r) {
    if (r - l == 1) return {{gaps[l].w}, gaps[l].p, gaps[l].q};
    int mid = (l + r) / 2;
    Seg a = merge(l, mid), b = merge(mid, r);
    int q = min(a.q, b.q);
    vector<ll> lp = mul(a.poly, binom(a.q - q)), rp = mul(b.poly, binom(b.q - q));
    int shift = b.p - a.p;
    vector<ll> out(max(lp.size(), rp.size() + shift), 0);
    for (size_t i = 0; i < lp.size(); i++) out[i] = lp[i];
    for (size_t i = 0; i < rp.size(); i++) out[i + shift] = (out[i + shift] + rp[i]) % MOD;
    return {out, a.p, q};
}
int main() {
    scanf("%d", &n);
    vector<pair<ll, int>> ev;
    for (int i = 0; i < n; i++) { ll a, b; scanf("%lld %lld", &a, &b); ev.push_back({a, 0}); ev.push_back({b, 1}); }
    sort(ev.begin(), ev.end());
    fact.assign(n + 1, 1), ifact.assign(n + 1, 1);
    for (int i = 1; i <= n; i++) fact[i] = fact[i - 1] * i % MOD;
    ifact[n] = pw(fact[n], MOD - 2);
    for (int i = n; i > 0; i--) ifact[i - 1] = ifact[i] * i % MOD;
    int p = 0, q = 0;
    for (size_t j = 1; j < ev.size(); j++) {
        if (ev[j - 1].second == 0) q++; else p++, q--;
        ll dv = ev[j].first - ev[j - 1].first;
        gaps.push_back({p, q, dv > 0 ? dv % MOD * pw(2, n - q) % MOD : 0});
    }
    Seg top = merge(0, gaps.size());
    vector<ll> diff = mul(top.poly, binom(top.q));
    diff.insert(diff.begin(), top.p, 0);
    ll run = 0;
    for (int i = 0; i < n; i++) {
        run = (run + (i == 0 ? pw(2, n) * (ev[0].first % MOD) % MOD : 0) + (i < (int)diff.size() ? diff[i] : 0)) % MOD;
        printf("%lld\n", run);
    }
}
