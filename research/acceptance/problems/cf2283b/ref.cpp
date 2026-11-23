#include <bits/stdc++.h>
using namespace std;
int main() {
    int t;
    if (scanf("%d", &t) != 1) return 0;
    while (t--) {
        int n;
        if (scanf("%d", &n) != 1) return 0;
        vector<int> a(n);
        for (auto &x : a) if (scanf("%d", &x) != 1) return 0;
        int best = 0;
        for (int v : a) {
            int c = 0;
            for (int x : a) c += x % v == 0;
            best = max(best, c);
        }
        printf("%d\n", n - best);
    }
}
