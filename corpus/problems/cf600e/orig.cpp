// Codeforces 600E (Lomsat gelral): small-to-large merge of colour counts, iterative DFS.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    scanf("%d", &n);
    vector<int> c(n + 1);
    for (int i = 1; i <= n; i++) scanf("%d", &c[i]);
    vector<vector<int>> adj(n + 1);
    for (int i = 0; i < n - 1; i++) {
        int x, y;
        scanf("%d %d", &x, &y);
        adj[x].push_back(y);
        adj[y].push_back(x);
    }
    vector<int> par(n + 1, 0), order;
    order.reserve(n);
    vector<int> st = {1};
    par[1] = -1;
    while (!st.empty()) {
        int v = st.back();
        st.pop_back();
        order.push_back(v);
        for (int u : adj[v]) if (u != par[v]) par[u] = v, st.push_back(u);
    }
    vector<map<int, int>*> cnt(n + 1);
    vector<int> top(n + 1);
    vector<long long> sum(n + 1), ans(n + 1);
    for (int v = 1; v <= n; v++) {
        cnt[v] = new map<int, int>();
        (*cnt[v])[c[v]] = 1;
        top[v] = 1;
        sum[v] = c[v];
    }
    for (int i = n - 1; i >= 0; i--) {
        int v = order[i];
        ans[v] = sum[v];
        if (par[v] <= 0) continue;
        int p = par[v];
        if (cnt[v]->size() > cnt[p]->size()) swap(cnt[v], cnt[p]), swap(top[v], top[p]), swap(sum[v], sum[p]);
        for (auto [col, k] : *cnt[v]) {
            int x = ((*cnt[p])[col] += k);
            if (x > top[p]) top[p] = x, sum[p] = col;
            else if (x == top[p]) sum[p] += col;
        }
        delete cnt[v];
    }
    for (int v = 1; v <= n; v++) printf("%lld%c", ans[v], v == n ? '\n' : ' ');
}
