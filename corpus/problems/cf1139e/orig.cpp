// Codeforces 1139E (Maximize Mex): students rejoin in reverse; potentials 0, 1, 2, ... are
// matched to distinct clubs by Kuhn's augmenting paths, and the mex only grows backwards.
#include <bits/stdc++.h>
using namespace std;
const int V = 5005;
vector<int> adj[V];
int matchClub[V], seen[V], stamp_ = 0;
bool aug(int p) {
    for (int c : adj[p]) {
        if (seen[c] == stamp_) continue;
        seen[c] = stamp_;
        if (matchClub[c] < 0 || aug(matchClub[c])) { matchClub[c] = p; return true; }
    }
    return false;
}
int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<int> p(n + 1), c(n + 1);
    for (int i = 1; i <= n; i++) scanf("%d", &p[i]);
    for (int i = 1; i <= n; i++) scanf("%d", &c[i]);
    int d;
    scanf("%d", &d);
    vector<int> k(d), gone(n + 1, 0);
    for (auto& x : k) scanf("%d", &x), gone[x] = 1;
    memset(matchClub, -1, sizeof matchClub);
    for (int i = 1; i <= n; i++) if (!gone[i]) adj[p[i]].push_back(c[i]);
    vector<int> ans(d);
    int mex = 0;
    for (int day = d - 1; day >= 0; day--) {
        while (mex < 5000) {
            stamp_++;
            if (!aug(mex)) break;
            mex++;
        }
        ans[day] = mex;
        adj[p[k[day]]].push_back(c[k[day]]);
    }
    for (int x : ans) printf("%d\n", x);
}
