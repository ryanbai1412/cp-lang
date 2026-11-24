// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int p, k, m;
    cin >> p >> k;
    string s, line, w;
    for (int i = 0; i < p; i++) cin >> line, s += line;
    int n = s.size();
    // end[i]: where the shortest word starting at i ends (one word per starting letter)
    vector<int> end(n, INT_MAX);
    cin >> m;
    for (int t = 0; t < m; t++) {
        cin >> w;
        for (int i = 0; i + (int)w.size() <= n; i++)
            if (s.compare(i, w.size(), w) == 0) end[i] = min(end[i], i + (int)w.size() - 1);
    }
    // cnt[l][r]: the words counted in s[l..r]
    vector<vector<int>> cnt(n, vector<int>(n));
    for (int l = n - 1; l >= 0; l--)
        for (int r = l; r < n; r++) cnt[l][r] = (l < r ? cnt[l + 1][r] : 0) + (end[l] <= r);
    // f[i][j]: the best count of s[0..i) cut into j parts
    vector<vector<int>> f(n + 1, vector<int>(k + 1, INT_MIN));
    f[0][0] = 0;
    for (int i = 1; i <= n; i++)
        for (int j = 1; j <= min(i, k); j++)
            for (int c = j - 1; c < i; c++)
                if (f[c][j - 1] != INT_MIN) f[i][j] = max(f[i][j], f[c][j - 1] + cnt[c][i - 1]);
    printf("%d\n", f[n][k]);
}
