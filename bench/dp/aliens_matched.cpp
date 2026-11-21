// Matched C++ for `aliens`: the same binary search on the penalty, stopping
// once a penalty gives exactly k, around the same O(n) penalized DP.
#include <bits/stdc++.h>
using namespace std; typedef long long ll;
struct R { ll val; ll count; };
static bool better(R x, R y) { return x.val < y.val || (x.val == y.val && x.count < y.count); }
int n; ll k; vector<ll> a;
R solve(ll lam) {
    R out{0, 0}, open{0, 0}; bool started = false;
    for (ll x : a) {
        R fresh{out.val - x + lam, out.count + 1}, ext{open.val - x, open.count};
        open = started && better(ext, fresh) ? ext : fresh;
        started = true;
        if (better(open, out)) out = open;
    }
    return out;
}
int main() {
    scanf("%d %lld", &n, &k); a.resize(n);
    for (auto& x : a) scanf("%lld", &x);
    ll lo = 0, hi = 2000000, ans = LLONG_MIN;
    while (lo < hi) {
        ll mid = lo + (hi - lo) / 2; R at = solve(mid);
        if (at.count == k) { ans = at.val - mid * k; break; }
        if (at.count < k) hi = mid; else lo = mid + 1;
    }
    if (ans == LLONG_MIN) ans = solve(lo).val - lo * k;
    printf("%lld\n", -ans);
}
