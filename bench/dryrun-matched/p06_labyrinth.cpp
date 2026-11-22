#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, m;
    cin >> n >> m;
    vector<string> g(n);
    for (auto& s : g) cin >> s;
    int s = 0, t = 0;
    for (int i = 0; i < n; i++)
        for (int j = 0; j < m; j++) {
            if (g[i][j] == 'A') s = i * m + j;
            if (g[i][j] == 'B') t = i * m + j;
        }
    const int dr[] = {-1, 0, 1, 0}, dc[] = {0, 1, 0, -1};
    const char name[] = "URDL";
    vector<int> back(n * m, -1), q{s};
    vector<char> via(n * m);
    back[s] = s;
    for (size_t h = 0; h < q.size(); h++) {
        int r = q[h] / m, c = q[h] % m;
        for (int k = 0; k < 4; k++) {
            int a = r + dr[k], b = c + dc[k];
            if (a < 0 || b < 0 || a >= n || b >= m || g[a][b] == '#' || back[a * m + b] >= 0) continue;
            back[a * m + b] = q[h], via[a * m + b] = name[k];
            q.push_back(a * m + b);
        }
    }
    if (back[t] < 0) {
        cout << "NO\n";
        return 0;
    }
    string path;
    for (int v = t; v != s; v = back[v]) path += via[v];
    reverse(path.begin(), path.end());
    cout << "YES\n" << path.size() << '\n' << path << '\n';
}
