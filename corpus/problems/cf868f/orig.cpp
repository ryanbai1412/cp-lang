// Codeforces 868F (Yet Another Minimization Problem): k layers of divide-and-conquer DP;
// a segment's cost comes from a two-pointer window that moves like Mo's algorithm.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
int n, k, a[100005], cnt[100005], L = 1, R = 0;
ll cur = 0, prv[100005], nxt[100005];
ll cost(int l, int r) {
    while (R < r) cur += cnt[a[++R]]++;
    while (L > l) cur += cnt[a[--L]]++;
    while (R > r) cur -= --cnt[a[R--]];
    while (L < l) cur -= --cnt[a[L++]];
    return cur;
}
void solve(int lo, int hi, int optlo, int opthi) {
    if (lo > hi) return;
    int mid = (lo + hi) / 2, best = optlo;
    ll bv = LLONG_MAX;
    for (int j = optlo; j <= min(mid - 1, opthi); j++) {
        ll v = prv[j] + cost(j + 1, mid);
        if (v < bv) bv = v, best = j;
    }
    nxt[mid] = bv;
    solve(lo, mid - 1, optlo, best);
    solve(mid + 1, hi, best, opthi);
}
int main() {
    scanf("%d %d", &n, &k);
    for (int i = 1; i <= n; i++) scanf("%d", &a[i]);
    for (int i = 1; i <= n; i++) prv[i] = cost(1, i);
    for (int layer = 2; layer <= k; layer++) {
        for (int i = 0; i < layer; i++) nxt[i] = LLONG_MAX / 4;
        solve(layer, n, layer - 1, n - 1);
        for (int i = 0; i <= n; i++) prv[i] = nxt[i];
    }
    printf("%lld\n", prv[n]);
}
