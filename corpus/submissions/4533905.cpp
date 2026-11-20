#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
		 << ": ";                                          \
	dbgh(__VA_ARGS__)
#else
#define cerr   \
	if (false) \
	cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

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

	bool conn(int u, int v) {
		return find(u) == find(v);
	}
};

enum Type {
	// person below me is x
	Below,
	// person above me is x
	Above,
	Other
};

struct A {
	string poster, msg;
	Type type;
	int ind;
};

void solve(const vector<A>& arr) {
	int n = sz(arr);
	DSU dsu(n);
	int lleft[n], lright[n];
	memset(lleft, -1, sizeof(lleft));
	memset(lright, -1, sizeof(lright));
	auto ca = [&](int a, int b) -> bool {
		return !dsu.conn(a, b) && lleft[b] == -1 && lright[a] == -1;
	};
	auto after = [&](int a, int b) -> void {
		dbg(a, b);
		assert(ca(a, b));
		lleft[b] = a;
		lright[a] = b;
		dsu.merge(a, b);
	};
	auto cnt = [&](int a, int b) -> int {
		int ans = 0;
		if (arr[a].type == Below && arr[a].msg == arr[b].poster) {
			ans++;
		}
		if (arr[b].type == Above && arr[a].poster == arr[b].msg) {
			ans++;
		}
		return ans;
	};
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && cnt(i, j) == 2) {
					after(i, j);
				}
			}
		}
	}
	// poster = msg and person above me has open slot
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && arr[i].msg == arr[i].poster && arr[i].type == Below && arr[j].type != Above &&
					arr[i].msg == arr[j].poster) {
					after(i, j);
				}
				if (ca(j, i) && arr[i].msg == arr[i].poster && arr[i].type == Above && arr[j].type != Below &&
					arr[i].msg == arr[j].poster) {
					after(j, i);
				}
			}
		}
	}
	// person above me has open slot and cannot pair with me
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && arr[i].type == Below && arr[j].type != Above &&
					arr[i].msg == arr[j].poster && !cnt(j, i)) {
					after(i, j);
				}
				if (ca(j, i) && arr[i].type == Above && arr[j].type != Below &&
					arr[i].msg == arr[j].poster && !cnt(i, j)) {
					after(j, i);
				}
			}
		}
	}
	// person above me has open slot
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && arr[i].type == Below && arr[j].type != Above &&
					arr[i].msg == arr[j].poster) {
					after(i, j);
				}
				if (ca(j, i) && arr[i].type == Above && arr[j].type != Below &&
					arr[i].msg == arr[j].poster) {
					after(j, i);
				}
			}
		}
	}
	// everything
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && arr[i].type == Below && arr[i].msg == arr[j].poster) {
					after(i, j);
				}
				if (ca(j, i) && arr[i].type == Above && arr[i].msg == arr[j].poster) {
					after(j, i);
				}
			}
		}
	}
	bool vis[n] {};
	vector<int> ans;
	for (int i = 0; i < n; i++) {
		if (vis[i]) {
			continue;
		}
		int u = i;
		while (lleft[u] != -1) {
			u = lleft[u];
		}
		while (u != -1) {
			ans.push_back(u);
			vis[u] = true;
			u = lright[u];
		}
	}
	int cans = 0;
	for (int i = 0; i < n - 1; i++) {
		cans += cnt(ans[i], ans[i + 1]);
	}
	cout << cans << endl;
	for (int i = 0; i < n; i++) {
		cout << ans[i] + 1 << " \n"[i == n - 1];
	}
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
	}
	vector<A> arr(m);
	for (int i = 0; i < m; i++) {
		string a, b, c;
		cin >> a >> b >> c;
		Type type = Other;
		if (c == "loushang") {
			type = Below;
		} else if (c == "louxia") {
			type = Above;
		}
		arr[i] = {a, b, type, i};
		cerr << i << ": ";
		if (arr[i].type == Other) {
			cerr << arr[i].poster << endl;
		} else {
			cerr << arr[i].poster << " " << arr[i].msg << " ";
			if (arr[i].type == Above) {
				cerr << "above";
			} else {
				cerr << "below";
			}
			cerr << endl;
		}
	}
	solve(arr);
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}