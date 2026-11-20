#include <bits/stdc++.h>
using namespace std;

int main() {
    int n, k;
    cin >> n >> k;
    string s;
    cin >> s;
    
    vector<string> grid(n, string(n, '.'));
    int r = n / 2, c = n / 2; // center
    
    for (char ch : s) {
        if (ch == 'L') { if (c > 0) c--; }
        else if (ch == 'R') { if (c < n-1) c++; }
        else if (ch == 'U') { if (r > 0) r--; }
        else if (ch == 'D') { if (r < n-1) r++; }
        else {
            // Paint all cells with Manhattan distance < k from (r, c)
            for (int i = 0; i < n; i++) {
                for (int j = 0; j < n; j++) {
                    if (abs(i - r) + abs(j - c) < k) {
                        grid[i][j] = ch;
                    }
                }
            }
        }
    }
    
    for (int i = 0; i < n; i++) {
        cout << grid[i] << "\n";
    }
    return 0;
}