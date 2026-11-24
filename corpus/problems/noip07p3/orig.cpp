// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
using i128 = __int128;
int main() {
    int n, m;
    cin >> n >> m;
    i128 total = 0;
    vector<long long> a(m);
    // g[l][r]: the best score of a[l..r] when its first take is worth 2^1
    vector<vector<i128>> g(m + 1, vector<i128>(m + 1));
    for (int row = 0; row < n; row++) {
        for (auto& x : a) cin >> x;
        for (int len = 1; len <= m; len++)
            for (int l = 0; l + len <= m; l++) {
                int r = l + len - 1;
                i128 inner_l = len > 1 ? g[l + 1][r] : 0, inner_r = len > 1 ? g[l][r - 1] : 0;
                g[l][r] = max(2 * (a[l] + inner_l), 2 * (a[r] + inner_r));
            }
        total += g[0][m - 1];
    }
    string s;
    do s += char('0' + (int)(total % 10)), total /= 10; while (total);
    reverse(s.begin(), s.end());
    puts(s.c_str());
}
