#include <bits/stdc++.h>
using namespace std;
// Erase E and keep x with sum(E) + x = total: the answer is one less than the
// smallest subset of [l, r] summing to total. k-subsets of an integer interval
// hit every sum between the k smallest and the k largest.
int main() {
    int t;
    scanf("%d", &t);
    while (t--) {
        int l, r;
        scanf("%d %d", &l, &r);
        int n = r - l + 1;
        long long total = 0;
        for (int v = l; v <= r; v++) total += v;
        long long lo = 0, hi = 0;
        for (int k = 1; k <= n; k++) {
            lo += l + k - 1, hi += r - k + 1;
            if (lo <= total && total <= hi) { printf("%d\n", k - 1); break; }
        }
    }
}
