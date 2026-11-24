// Codeforces 802 "Fake News (hard)": the sum over substrings p of occ(p)^2. In a suffix
// automaton each state holds len - link.len substrings that all occur endpos-count times.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
struct St { int len, link, nx[26]; };
int main() {
    int t;
    scanf("%d", &t);
    static char buf[100005];
    while (t--) {
        scanf("%s", buf);
        int n = strlen(buf);
        vector<St> s(2 * n + 2);
        vector<ll> cnt(2 * n + 2, 0);
        int sz = 1, last = 0;
        s[0].len = 0, s[0].link = -1;
        memset(s[0].nx, -1, sizeof s[0].nx);
        for (int i = 0; i < n; i++) {
            int ch = buf[i] - 'a', cur = sz++;
            s[cur].len = s[last].len + 1;
            memset(s[cur].nx, -1, sizeof s[cur].nx);
            cnt[cur] = 1;
            int p = last;
            while (p != -1 && s[p].nx[ch] == -1) s[p].nx[ch] = cur, p = s[p].link;
            if (p == -1) s[cur].link = 0;
            else {
                int q = s[p].nx[ch];
                if (s[p].len + 1 == s[q].len) s[cur].link = q;
                else {
                    int cl = sz++;
                    s[cl] = s[q];
                    s[cl].len = s[p].len + 1;
                    while (p != -1 && s[p].nx[ch] == q) s[p].nx[ch] = cl, p = s[p].link;
                    s[q].link = s[cur].link = cl;
                }
            }
            last = cur;
        }
        vector<int> order(sz);
        iota(order.begin(), order.end(), 0);
        sort(order.begin(), order.end(), [&](int a, int b) { return s[a].len > s[b].len; });
        ll ans = 0;
        for (int v : order) {
            if (v == 0) continue;
            cnt[s[v].link] += cnt[v];
            ans += (ll)(s[v].len - s[s[v].link].len) * cnt[v] * cnt[v];
        }
        printf("%lld\n", ans);
    }
}
