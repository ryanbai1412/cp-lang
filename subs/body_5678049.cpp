
void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    map<long, int> cnt;
    for (auto& a : arr) {
        cnt[a]++;
    }
    int q;
    cin >> q;
    long radd = 0, csum = 0;
    for (auto& a : arr) {
        csum += a;
    }
    while (q--) {
        string ty;
        cin >> ty;
        if (ty == "INFLATION") {
            int x;
            cin >> x;
            radd += x;
        } else {
            long u, v;
            cin >> u >> v;
            if (u != v) {
                int &ccnt = cnt[u - radd];
                cnt[v - radd] += ccnt;
                csum += (v - u) * ccnt;
                ccnt = 0;
            }
        }
        cout << csum + radd * n << endl;
    }
}
