// The accepted ioi00p1 solution's DP (corpus/submissions/4682427.cpp).
#include <bits/stdc++.h>
using namespace std;

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    string s;
    cin >> n >> s;
    static int dp[2][5000];
    int cind = 0, pind = 1;
    for (int i = n - 1; i >= 0; i--) {
        swap(cind, pind);
        dp[cind][i] = 0;
        if (i + 1 < n) dp[pind][i] = 0;
        for (int j = i + 1; j < n; j++) {
            dp[cind][j] = 1 + min(dp[pind][j], dp[cind][j - 1]);
            if (s[i] == s[j]) dp[cind][j] = min(dp[cind][j], dp[pind][j - 1]);
        }
    }
    cout << dp[cind][n - 1] << "\n";
}
