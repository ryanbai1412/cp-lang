// not my code
#include <bits/stdc++.h>
using namespace std;
using pii=pair<int,int>;

struct DSU {
    vector<int> p;

    DSU(int n) : p(n, -1) {}

    int find(int u) {
        return p[u] < 0 ? u : (p[u] = find(p[u]));
    }

    bool merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            return false;
        }
        if (p[u] < p[v]) {
            swap(u, v);
        }
        p[v] += p[u];
        p[u] = v;
        return true;
    }
};

int mst(vector<vector<pii>> &graph) {
    vector<array<int, 3>> edges;
    for (int i = 0; i < graph.size(); i++) {
        for (auto& [ v, w] : graph[i]) {
            edges.push_back({w, v, i});
        }
    }
    sort(begin(edges), end(edges));

    int ans = 0;
    DSU dsu(graph.size());
    for (auto& [w, u, v] : edges) {
        if (dsu.merge(u, v)) {
            ans += w;
        }
    }

    return ans;
}

struct edge {
    vector<int> pens;
    int cost;
};

int main() {
    int nm;
    cin >> nm;
    int n=1e3+1;
    vector<vector<edge>> edges(n,vector<edge>(n));
    for (int i=0; i<nm; ++i) {
        int m;
        cin >> m;
        vector<int> v(m);
        for (auto &item : v) {
            cin >> item;
        }
        vector<int> costs(m);
        for (auto &item : costs) cin >> item;
        for (int j=0; j<m; ++j) {
            int a=v[j];
            int b=v[(j+1)%m];
            edges[a][b].pens.push_back(i+1);
            edges[a][b].cost=costs[j];
            edges[b][a].pens.push_back(i+1);
            edges[b][a].cost=costs[j];
        }
    }
    vector<vector<pii>> graph(n+1);
    for (int i=0; i<n; ++i) {
        for (int j=0; j<n; ++j) {
            if (edges[i][j].pens.size()==2) {
                int a=edges[i][j].pens[0];
                int b=edges[i][j].pens[1];
                graph[a].push_back({b,edges[i][j].cost});
                graph[b].push_back({a,edges[i][j].cost});
            }
        }
    }
    int ans1=mst(graph);
    for (int i=0; i<n; ++i) {
        for (int j=0; j<n; ++j) {
            if (edges[i][j].pens.size()==1) {
                int a=edges[i][j].pens[0];
                graph[a].push_back({0,edges[i][j].cost});
                graph[0].push_back({a,edges[i][j].cost});
            }
        }
    }
    int ans2=mst(graph);
    cout << min(ans1,ans2) << endl;
}