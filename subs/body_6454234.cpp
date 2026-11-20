
int mod3(int x) {
	x %= 3;
	if (x < 0) {
		return x + 3;
	}
	return x;
}
struct DSU {
	vector<int> p, add;
	DSU(int n) : p(n, -1), add(n, 0) {}
	pair<int, int> find(int u) {
		if (p[u] < 0) {
			return {u, 0};
		}
		auto [up, ua] = find(p[u]);
		p[u] = up;
		add[u] += ua;
		return {p[u], add[u]};
	}
	bool merge(int iu, int iv, int x) {
		auto [u, ua] = find(iu);
		auto [v, va] = find(iv);
		dbg(u, v, ua, va);
		if (u == v) {
			if (mod3(ua - va) != x) {
				return false;
			}
			return true;
		}
		dbg(u, v, x);
		if (p[u] < p[v]) {
			swap(u, v);
			swap(ua, va);
			x = -x;
		}
		p[v] += p[u];
		p[u] = v;
		add[u] = va - ua + x;
		return true;
	}
};
void solve() {
	int n, q;
	cin >> n >> q;
	DSU dsu(n);
	int ans = 0;
	while (q--) {
		int ty, u, v;
		cin >> ty >> u >> v;
		u--;
		v--;
		if (u >= n || v >= n) {
			ans++;
			continue;
		} else if (ty == 2 && u == v) {
			ans++;
			continue;
		}
		if (ty == 1) {
			ans += !dsu.merge(u, v, 0);
		} else {
			ans += !dsu.merge(u, v, 1);
		}
	}
	cout << ans << endl;
}
