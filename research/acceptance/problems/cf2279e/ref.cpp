// 2279E Save Me!: a zero sum over index set S costs sum over q in (min S, max S] of
// (1 if q in S else 2); the answer is at most 49 when n > 25. Meet in the middle at each
// split i: left sets end at i with cost <= 25 (hash table of min cost per xor), right sets
// start after i with cost <= 25.
#include <bits/stdc++.h>
using namespace std;

const int CAP = 25, LOG = 19, SZ = 1 << LOG;
static int key[SZ];
static unsigned char val[SZ];
int n, best;
vector<int> a;

inline unsigned slot(int k) { return (unsigned)(k * 2654435761u) >> (32 - LOG); }
void put(int k, int c) {
    unsigned h = slot(k);
    while (key[h] && key[h] != k + 1) h = (h + 1) & (SZ - 1);
    if (!key[h] || c < val[h]) key[h] = k + 1, val[h] = c;
}
int get(int k) {
    unsigned h = slot(k);
    while (key[h]) {
        if (key[h] == k + 1) return val[h];
        h = (h + 1) & (SZ - 1);
    }
    return -1;
}
void left(int p, int acc, int x) {  // positions (p, i] decided, acc = their cost
    put(x ^ a[p], acc);
    if (p == 0) return;
    if (acc + 1 <= CAP) left(p - 1, acc + 1, x ^ a[p]);
    if (acc + 2 <= CAP) left(p - 1, acc + 2, x);
}
void right(int p, int acc, int x) {  // positions [i+1, p) decided, acc = their cost
    if (acc + 1 >= best) return;
    int c = get(x ^ a[p]);
    if (c >= 0) best = min(best, c + acc + 1);
    if (p + 1 == n) return;
    if (acc + 2 <= CAP) right(p + 1, acc + 1, x ^ a[p]);
    if (acc + 3 <= CAP) right(p + 1, acc + 2, x);
}
int main() {
    int t;
    scanf("%d", &t);
    while (t--) {
        scanf("%d", &n);
        a.assign(n, 0);
        for (auto& v : a) scanf("%d", &v);
        if (count(a.begin(), a.end(), 0)) { puts("0"); continue; }
        best = INT_MAX;
        for (int i = 0; i + 1 < n; i++) {
            memset(key, 0, sizeof key);
            left(i, 0, 0);
            right(i + 1, 0, 0);
        }
        printf("%d\n", best == INT_MAX ? -1 : best);
    }
}
