// sdoi17r2p2 (SDOI 2017 Apple Tree) reference. Extending the deepest picked node down to a
// leaf with one apple per node keeps t - h, so some root-to-leaf chain x has one free apple
// per node and at most k other apples obey the parent rule. Split node i into its first apple
// and a child holding the other a_i - 1. A DFS where a child's table starts as a copy of its
// parent's and merges back with 1..cnt of its apples (sliding-window max) leaves, at leaf x,
// the best use of everything left of the chain (children in order) or right of it (children
// reversed), the chain uncharged. Answer: max over x of chain value + best split of k.
// O(nk) time, about n (k + 1) ints of tables.
#include <bits/stdc++.h>
using namespace std;

int n, k;
vector<int> cnt, val;
vector<vector<int>> kids;
vector<vector<int>> pool;
vector<int> freeIds;

int alloc(const vector<int>& from) {
    int id;
    if (freeIds.empty()) id = pool.size(), pool.push_back(from);
    else id = freeIds.back(), freeIds.pop_back(), pool[id] = from;
    return id;
}

void merge(vector<int>& up, const vector<int>& f, int c, int v) {
    deque<int> dq;  // indices i in [j - c, j - 1] by decreasing f[i] - i v
    for (int j = 1; j <= k; j++) {
        int i = j - 1;
        while (!dq.empty() && f[dq.back()] - (long long)dq.back() * v <= f[i] - (long long)i * v) dq.pop_back();
        dq.push_back(i);
        while (dq.front() < j - c) dq.pop_front();
        int b = dq.front();
        up[j] = max<long long>(up[j], f[b] + (long long)(j - b) * v);
    }
}

// onLeaf(x, table): forward passes the finished table, reverse the starting one.
template <class Hook>
void pass(bool reversed, bool atStart, Hook onLeaf) {
    pool.clear(), freeIds.clear();
    struct Fr { int u, ci, buf; };
    vector<Fr> st = {{0, 0, alloc(vector<int>(k + 1, 0))}};
    auto leaf = [&](int u) { return u < n && kids[u].size() <= 1 && (kids[u].empty() || kids[u][0] >= n); };
    if (atStart && leaf(0)) onLeaf(0, pool[st[0].buf]);
    while (!st.empty()) {
        Fr& fr = st.back();
        int u = fr.u;
        if (fr.ci < (int)kids[u].size()) {
            int idx = fr.ci++;
            int v = kids[u][reversed ? kids[u].size() - 1 - idx : idx];
            int b = alloc(pool[fr.buf]);
            if (atStart && leaf(v)) onLeaf(v, pool[b]);
            st.push_back({v, 0, b});
            continue;
        }
        int b = fr.buf;
        st.pop_back();
        if (!atStart && leaf(u)) onLeaf(u, pool[b]);
        if (!st.empty()) merge(pool[st.back().buf], pool[b], cnt[u], val[u]);
        freeIds.push_back(b);
    }
}

int main() {
    int Q;
    scanf("%d", &Q);
    while (Q--) {
        scanf("%d %d", &n, &k);
        cnt.assign(n, 1), val.assign(n, 0), kids.assign(n, {});
        vector<int> par(n), a(n);
        vector<long long> chain(n);
        for (int i = 0; i < n; i++) {
            scanf("%d %d %d", &par[i], &a[i], &val[i]);
            par[i]--;
            chain[i] = val[i] + (i ? chain[par[i]] : 0);
        }
        for (int i = 0; i < n; i++)
            if (a[i] > 1) {
                kids[i].push_back(cnt.size());
                cnt.push_back(min(a[i] - 1, k)), val.push_back(val[i]), kids.push_back({});
            }
        for (int i = 1; i < n; i++) kids[par[i]].push_back(i);
        // forward: the split child comes first, so a chain node's extra apples sit on the left
        vector<vector<int>> left(n);
        pass(false, false, [&](int x, const vector<int>& f) { left[x] = f; });
        long long best = 0;
        pass(true, true, [&](int x, const vector<int>& g) {
            for (int j = 0; j <= k; j++) best = max(best, chain[x] + left[x][j] + g[k - j]);
            vector<int>().swap(left[x]);
        });
        printf("%lld\n", best);
    }
}
