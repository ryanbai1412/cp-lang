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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 2e5;

struct Fact {
	vector<int> mdiv;

	explicit Fact(int n) : mdiv(n + 1) {
		for (int i = 2; i <= n; i++) {
			if (!mdiv[i]) {
				mdiv[i] = i;
				if (n / i > i) {
					for (int j = i * i; j <= n; j += i) {
						if (!mdiv[j]) {
							mdiv[j] = i;
						}
					}
				}
			}
		}
	}

	vector<pair<int, int>> factorize(int x) {
		assert(x < sz(mdiv));
		vector<pair<int, int>> fact;
		while (x != 1) {
			int p = mdiv[x], exp = 0;
			while (x % p == 0) {
				x /= p;
				exp++;
			}
			fact.emplace_back(p, exp);
		}
		return fact;
	}
} fact(maxn);

int d, arr[maxn], ans[maxn];
vector<int> graph[maxn];

int f(int a, int b) {
	if (a == 1 && b == 1) {
		return 1;
	}
	auto facta = fact.factorize(a);
	auto factb = fact.factorize(b);
	int n = sz(facta), m = sz(factb);
	facta.emplace_back(1e9, 0);
	factb.emplace_back(1e9, 0);
	int i = 0, j = 0;
	int ans = 1;
	while (i < n || j < m) {
		int p = min(facta[i].first, factb[j].first), exp = 0;
		if (facta[i].first == p) {
			exp += facta[i].second;
			i++;
		}
		if (factb[j].first == p) {
			exp += factb[j].second;
			j++;
		}
		if (exp > d) {
			return 0;
		}
		if (exp <= d && exp % 2 == 1) {
				ans *= -1;
		}
	}
	return ans;
}

struct DS {
	int ans = 0;
	vector<int> cur;

	void update(int x) {
		cur.push_back(x);
		for (auto& a : cur) {
			ans += f(x, a);
		}
	}

	void merge_with(const DS& ds) {
		for (auto& a : ds.cur) {
			update(a);
		}
	}

	int compute() const { return ans; }

	static DS small_to_large(DS& a, DS& b) {
		if (sz(a.cur) < sz(b.cur)) {
			swap(a, b);
		}
		a.merge_with(b);
		return a;
	}
};

DS dfs(int u, int p = -1) {
	DS cur;
	cur.update(arr[u]);
	for (auto& v : graph[u]) {
		if (v != p) {
			auto x = dfs(v, u);
			DS::small_to_large(cur, x);
		}
	}
	ans[u] = cur.compute();
	return cur;
}

void solve() {
	int n;
	cin >> n >> d;
	dbg(f(4, 4), f(4, 15), f(15, 15));
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	dfs(0);
	for (int i = 0; i < n; i++) {
		cout << ans[i] << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}