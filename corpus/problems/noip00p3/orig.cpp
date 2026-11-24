// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
int n, best, ov[21][20], used[20];
string w[21];
void dfs(int i, int len) {
    best = max(best, len);
    for (int j = 0; j < n; j++)
        if (used[j] < 2 && ov[i][j] > 0) {
            used[j]++;
            dfs(j, len + (int)w[j].size() - ov[i][j]);
            used[j]--;
        }
}
int main() {
    cin >> n;
    for (int i = 0; i <= n; i++) cin >> w[i];  // w[n]: the start letter
    for (int i = 0; i <= n; i++)
        for (int j = 0; j < n; j++) {
            // the shortest overlap that keeps both parts: shorter than both words (or the whole letter)
            int lim = i == n ? 1 : (int)min(w[i].size(), w[j].size()) - 1;
            ov[i][j] = 0;
            for (int k = 1; k <= lim && k <= (int)min(w[i].size(), w[j].size()); k++)
                if (w[i].compare(w[i].size() - k, k, w[j], 0, k) == 0) { ov[i][j] = k; break; }
        }
    dfs(n, 1);
    printf("%d\n", best == 1 && none_of(w, w + n, [](auto& s) { return s[0] == w[n][0]; }) ? 0 : best);
}
