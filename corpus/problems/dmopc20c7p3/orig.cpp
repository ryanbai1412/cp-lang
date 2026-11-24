// Sweep left to right with a queue of open strokes by start (official editorial): a drop in A
// ends the oldest strokes whose length is in [L, R]; then strokes longer than R, each costing a
// second stroke (two strokes of lengths in [L, R] cover any length up to N); younger strokes
// cannot end, so the case is impossible.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int t;
    if (scanf("%d", &t) != 1) return 0;
    vector<long long> a;
    deque<pair<int, long long>> open;
    while (t--) {
        int n, l, r;
        scanf("%d %d %d", &n, &l, &r);
        a.assign(n + 2, 0);
        for (int i = 1; i <= n; i++) scanf("%lld", &a[i]);
        open.clear();
        long long overlong = 0, ans = 0;
        bool ok = true;
        for (int i = 1; i <= n + 1 && ok; i++) {
            while (!open.empty() && i - open.front().first > r) {
                overlong += open.front().second;
                open.pop_front();
            }
            if (a[i] > a[i - 1]) {
                ans += a[i] - a[i - 1];
                open.emplace_back(i, a[i] - a[i - 1]);
            } else if (a[i] < a[i - 1]) {
                long long need = a[i - 1] - a[i];
                while (need && !open.empty() && i - open.front().first >= l) {
                    long long take = min(need, open.front().second);
                    need -= take;
                    if (!(open.front().second -= take)) open.pop_front();
                }
                long long take = min(need, overlong);
                need -= take, overlong -= take, ans += take;
                if (need) ok = false;
            }
        }
        printf("%lld\n", ok ? ans : -1);
    }
}
