#include <bits/stdc++.h>
using namespace std;
static char buf[1 << 25];
int bl, bp;
int rd() {
    while (bp < bl && (buf[bp] < '0' || buf[bp] > '9')) bp++;
    int x = 0;
    while (bp < bl && buf[bp] >= '0' && buf[bp] <= '9') x = x * 10 + buf[bp++] - '0';
    return x;
}
int main() {
    bl = fread(buf, 1, sizeof buf, stdin);
    int t = rd();
    vector<int> head, nxt, to, comp, st;
    string out;
    while (t--) {
        int n = rd();
        head.assign(n + 1, -1); nxt.resize(2 * n); to.resize(2 * n);
        for (int e = 0; e < n - 1; e++) {
            int u = rd(), v = rd();
            to[2 * e] = v; nxt[2 * e] = head[u]; head[u] = 2 * e;
            to[2 * e + 1] = u; nxt[2 * e + 1] = head[v]; head[v] = 2 * e + 1;
        }
        int ans = n + 1;
        if (n % 2) {
            // n-1 black is the most possible for odd n; k = n, n+1, n+2 leave x = n, (n+1)/2, 1 unpaired.
            // x turns gray iff some pair (v, k-v) lies in different components of the tree without x.
            for (int k : {n, n + 1, n + 2}) {
                int x = k == n ? n : k == n + 1 ? (n + 1) / 2 : 1;
                comp.assign(n + 1, 0);
                int c = 0;
                for (int e = head[x]; e != -1; e = nxt[e]) {
                    int s = to[e]; c++;
                    comp[s] = c; st.assign(1, s);
                    while (!st.empty()) {
                        int u = st.back(); st.pop_back();
                        for (int f = head[u]; f != -1; f = nxt[f])
                            if (to[f] != x && !comp[to[f]]) comp[to[f]] = c, st.push_back(to[f]);
                    }
                }
                bool gray = false;
                for (int v = 1; v <= n && !gray; v++)
                    if (v != x && comp[v] != comp[k - v]) gray = true;
                if (gray) { ans = k; break; }
            }
        }
        out += to_string(ans); out += '\n';
    }
    fwrite(out.data(), 1, out.size(), stdout);
}
