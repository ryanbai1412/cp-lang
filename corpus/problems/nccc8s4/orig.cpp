// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
const int N = 53;
bool g[N][N];
int state[N], found;
void dfs(int u) {
    if (state[u]) {
        if (state[u] == 1) found = 1;
        return;
    }
    state[u] = 1;
    for (int v = 0; v < N; v++)
        if (g[u][v]) dfs(v);
    state[u] = 2;
}
int main() {
    int m;
    scanf("%d", &m);
    for (int i = 0; i < m; i++) {
        int c[4];
        for (int j = 0; j < 4; j++) scanf("%d", &c[j]), c[j] += 26;
        for (int j = 0; j < 4; j++)
            for (int k = 0; k < 4; k++)
                if (j != k && c[j] != 26 && c[k] != 26) g[c[j]][52 - c[k]] = true;
    }
    for (int i = 0; i < N; i++) dfs(i);
    printf("%d\n", found);
}
