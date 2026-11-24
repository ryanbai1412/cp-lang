// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const int B = 60;
ll f[B + 1][B + 1][B + 1];
int main() {
    f[0][0][0] = 1;
    for (int k = 1; k <= B; k++)
        for (int a = 0; a <= k; a++)
            for (int b = 0; b <= a; b++) {
                ll best = 0;
                for (int c = b; c <= a; c++) {
                    ll left = c > k - 1 ? 0 : f[k - 1][min(a, k - 1)][c];
                    ll right = (c == 0 || b - 1 > k - 1) ? 0 : f[k - 1][min(c - 1, k - 1)][max(0, b - 1)];
                    best = max(best, left + right);
                }
                f[k][a][b] = best;
            }
    int t;
    cin >> t;
    while (t--) {
        ll l, r;
        cin >> l >> r;
        vector<pair<int, int>> blocks;
        for (ll lo = l, hi = r + 1; lo < hi;) {
            int k = 0;
            while (k < B && !(lo >> k & 1) && lo + (2LL << k) <= hi) k++;
            blocks.push_back({__builtin_popcountll(lo >> k), k});
            lo += 1LL << k;
        }
        vector<ll> dp(B + 1, 0);
        for (auto [o, k] : blocks) {
            vector<ll> nx = dp;
            int top = min(B, o + k);
            for (int b = 0; b <= top; b++)
                for (int a = max(b, o); a <= top; a++) nx[b] = max(nx[b], dp[a] + f[k][a - o][max(0, b - o)]);
            for (int b = B - 1; b >= 0; b--) nx[b] = max(nx[b], nx[b + 1]);
            dp = nx;
        }
        cout << dp[0] << "\n";
    }
}
