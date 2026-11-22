#include <bits/stdc++.h>
using namespace std;
int main() {
    int h, w;
    cin >> h >> w;
    vector<string> g(h);
    for (auto& row : g) cin >> row;
    int best = 0;
    for (int r = 0; r < h; r++) {
        int run = 0;
        for (int c = 0; c < w; c++) {
            if (g[r][c] == '.') {
                run++;
                best = max(best, run);
            } else {
                run = 0;
            }
        }
    }
    cout << best << "\n";
}
