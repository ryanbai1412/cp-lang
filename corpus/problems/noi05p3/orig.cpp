// NOI 2005 Wisdom Beads reference: exact cover by backtracking. The first empty cell in row
// order must be the first cell (in the same order) of whichever piece covers it, so each
// unused piece is tried in each of its distinct orientations anchored there.
#include <bits/stdc++.h>
using namespace std;

const char* FIG[10] = {"B",       "BK",       "BKK",       "BJKK",       "JJJDD",
                       "GJGDDC",  "GGGCCCI",  "EEEHHIIA",  "ELHHHIAAF",  "ELLLLIFFFF"};
vector<vector<pair<int, int>>> shapes[12];
string g[10];
bool used[12];

bool on(int i, int j) { return i >= 0 && i < 10 && j >= 0 && j <= i; }

bool dfs(int at) {
    while (at < 55) {
        int i = 0;
        while ((i + 1) * (i + 2) / 2 <= at) i++;
        if (g[i][at - i * (i + 1) / 2] != '.') at++;
        else break;
    }
    if (at == 55) return true;
    int i = 0;
    while ((i + 1) * (i + 2) / 2 <= at) i++;
    int j = at - i * (i + 1) / 2;
    for (int p = 0; p < 12; p++) {
        if (used[p]) continue;
        for (auto& o : shapes[p]) {
            int di = i - o[0].first, dj = j - o[0].second;
            bool fits = true;
            for (auto [x, y] : o) fits &= on(x + di, y + dj) && g[x + di][y + dj] == '.';
            if (!fits) continue;
            for (auto [x, y] : o) g[x + di][y + dj] = 'A' + p;
            used[p] = true;
            if (dfs(at + 1)) return true;
            used[p] = false;
            for (auto [x, y] : o) g[x + di][y + dj] = '.';
        }
    }
    return false;
}

int main() {
    for (int p = 0; p < 12; p++) {
        vector<pair<int, int>> cells;
        for (int i = 0; i < 10; i++)
            for (int j = 0; j <= i; j++)
                if (FIG[i][j] == 'A' + p) cells.push_back({i, j});
        set<vector<pair<int, int>>> seen;
        for (int f = 0; f < 2; f++) {
            auto cs = cells;
            if (f) for (auto& [x, y] : cs) y = -y;
            for (int r = 0; r < 4; r++) {
                for (auto& [x, y] : cs) tie(x, y) = make_pair(y, -x);
                int mx = INT_MAX, my = INT_MAX;
                for (auto [x, y] : cs) mx = min(mx, x), my = min(my, y);
                vector<pair<int, int>> n;
                for (auto [x, y] : cs) n.push_back({x - mx, y - my});
                sort(n.begin(), n.end());
                seen.insert(n);
            }
        }
        shapes[p].assign(seen.begin(), seen.end());
    }
    int free = 0, need = 0;
    for (int i = 0; i < 10; i++) {
        cin >> g[i];
        for (char c : g[i])
            if (c == '.') free++;
            else used[c - 'A'] = true;
    }
    for (int p = 0; p < 12; p++)
        if (!used[p]) need += shapes[p][0].size();
    if (need == free && dfs(0))
        for (auto& r : g) cout << r << '\n';
    else
        cout << "No solution\n";
}
