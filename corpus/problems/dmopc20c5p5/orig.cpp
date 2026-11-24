// Row i is the Thue-Morse word shifted by i (cell (i, j) = t_{i+j}). An ugly rectangle in
// shifted copies of one word forces two overlapping equal factors, and Thue-Morse is
// overlap-free, so the colouring has no ugly rectangle at all (official editorial).
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    string row(m, 'B');
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) row[j] = __builtin_popcount(i + j) & 1 ? 'Y' : 'B';
        puts(row.c_str());
    }
}
