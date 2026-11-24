// sdoi17r1p5 (SDOI 2017 Coin Game) reference. Let N be the probability mass of the
// unfinished sequences, scaled by 2^-m, and x_j the chance student j wins. Appending s_i to an
// unfinished sequence ends the game at the first point some s_j appears, which overlaps the
// start of s_i by k characters (a suffix of s_j of length k equals a prefix of s_i):
//   N = sum_j x_j sum_{k overlap} 2^(k-m),  sum_j x_j = 1.
// Gaussian elimination on the n + 1 unknowns, overlaps from prefix functions: O(n^2 m + n^3).
#include <bits/stdc++.h>
using namespace std;

int main() {
    int n, m;
    cin >> n >> m;
    vector<string> s(n);
    for (auto& x : s) cin >> x;
    vector<double> pw(m + 1);
    pw[0] = 1;
    for (int k = 1; k <= m; k++) pw[k] = pw[k - 1] * 2;
    int N = n + 1;
    vector<vector<double>> A(N, vector<double>(N + 1, 0));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            string t = s[i] + "#" + s[j];
            int L = t.size();
            vector<int> p(L, 0);
            for (int q = 1; q < L; q++) {
                int k = p[q - 1];
                while (k && t[q] != t[k]) k = p[k - 1];
                if (t[q] == t[k]) k++;
                p[q] = k;
            }
            double c = 0;
            for (int k = p[L - 1]; k; k = p[k - 1]) c += pw[k] / pw[m];  // k = m only for i == j
            A[i][j] = c;
        }
        A[i][n] = -1;
    }
    for (int j = 0; j < n; j++) A[n][j] = 1;
    A[n][N] = 1;
    for (int col = 0; col < N; col++) {
        int piv = col;
        for (int r = col; r < N; r++)
            if (fabs(A[r][col]) > fabs(A[piv][col])) piv = r;
        swap(A[col], A[piv]);
        for (int r = 0; r < N; r++) {
            if (r == col || A[r][col] == 0) continue;
            double f = A[r][col] / A[col][col];
            for (int k = col; k <= N; k++) A[r][k] -= f * A[col][k];
        }
    }
    for (int i = 0; i < n; i++) printf("%.10f\n", A[i][N] / A[i][i]);
}
