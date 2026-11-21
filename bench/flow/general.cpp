#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
const int N = 10000;
int main() {
    vector<vector<int>> nb(N);
    for (int i = 0; i < 15000; i++) { int u = nxt(N), v = nxt(N); nb[u].push_back(v); nb[v].push_back(u); }
    vector<int> mate(N, -1); int size = 0;
    for (int x = 0; x < N; x++) { if (mate[x] != -1) continue;
        for (int y : nb[x]) if (y != x && mate[y] == -1) { mate[x] = y; mate[y] = x; size++; break; } }
    vector<int> parent(N), base(N); vector<char> inq(N), blossom(N), seen(N);
    for (int root = 0; root < N; root++) {
        if (mate[root] != -1) continue;
        for (int i = 0; i < N; i++) { parent[i] = -1; base[i] = i; inq[i] = 0; }
        inq[root] = 1; vector<int> queue{root}; size_t at = 0; int end = -1;
        while (at < queue.size() && end == -1) {
            int x = queue[at++];
            for (int y : nb[x]) {
                if (base[x] == base[y] || mate[x] == y) continue;
                if (y == root || (mate[y] != -1 && parent[mate[y]] != -1)) {
                    fill(seen.begin(), seen.end(), 0);
                    int a = x; while (true) { a = base[a]; seen[a] = 1; if (mate[a] == -1) break; a = parent[mate[a]]; }
                    int b = y; while (true) { b = base[b]; if (seen[b]) break; b = parent[mate[b]]; }
                    int top = b; fill(blossom.begin(), blossom.end(), 0);
                    for (auto [from, child] : {pair<int, int>{x, y}, pair<int, int>{y, x}}) {
                        int w = from, c = child;
                        while (base[w] != top) { blossom[base[w]] = 1; blossom[base[mate[w]]] = 1; parent[w] = c; c = mate[w]; w = parent[mate[w]]; } }
                    for (int i = 0; i < N; i++) if (blossom[base[i]]) { base[i] = top; if (!inq[i]) { inq[i] = 1; queue.push_back(i); } }
                } else if (parent[y] == -1) {
                    parent[y] = x;
                    if (mate[y] == -1) { end = y; break; }
                    inq[mate[y]] = 1; queue.push_back(mate[y]);
                }
            }
        }
        int y = end; if (y != -1) size++;
        while (y != -1) { int x = parent[y]; int next = mate[x]; mate[y] = x; mate[x] = y; y = next; }
    }
    printf("%d\n", size);
}
