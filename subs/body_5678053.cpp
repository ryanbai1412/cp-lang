
void solve() {
    int n;
    cin >> n;
    vector<int> path {0};
    for (int i = 1; i < n; i++) {
        int carr[i];
        for (auto& a : carr) {
            cin >> a;
        }
        bool is_adj[n] {};
        for (int j = 0; j < (i + 1) / 2; j++) {
            is_adj[carr[j]] = true;
        }
        if (is_adj[path[0]]) {
            path.insert(path.begin(), i);
        } else if (is_adj[path.back()]) {
            path.push_back(i);
        } else {
            for (int j = 0; j + 1 < sz(path); j++) {
                if (is_adj[path[j]] && is_adj[path[j + 1]]) {
                    path.insert(path.begin() + j + 1, i);
                    break;
                }
            }
        }
    }
    for (int i = 0; i < n; i++) {
        cout << path[i] << " \n"[i == n - 1];
    }
}
