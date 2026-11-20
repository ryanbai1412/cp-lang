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

struct custom_hash {
	static uint64_t splitmix64(uint64_t x) {
		// http://xorshift.di.unimi.it/splitmix64.c
		x += 0x9e3779b97f4a7c15;
		x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9;
		x = (x ^ (x >> 27)) * 0x94d049bb133111eb;
		return x ^ (x >> 31);
	}

	size_t operator()(uint64_t x) const {
		static const uint64_t FIXED_RANDOM =
			chrono::steady_clock::now().time_since_epoch().count();
		return splitmix64(x + FIXED_RANDOM);
	}
};

const int maxn = 2e5, bsize = 0;

template <int maxn>
struct Fact {
	int mdiv[maxn + 1] {};
	vector<pair<int, int>> cache[maxn + 1];

	Fact() {
		for (int i = 2; i <= maxn; i++) {
			if (!mdiv[i]) {
				mdiv[i] = i;
				if (long(i) * i <= maxn) {
					for (int j = i * i; j <= maxn; j += i) {
						if (!mdiv[j]) {
							mdiv[j] = i;
						}
					}
				}
			}
			cache[i] = factorize(i);
		}
	}

	vector<pair<int, int>> factorize(int x) {
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
};

Fact<maxn> fact;
int d, arr[maxn], siz[maxn], cache_cnt[maxn + 1], ans[maxn];
vector<int> graph[maxn], cache_ppows[maxn + 1];

int f(int a, int b) {
	if (a == 1 && b == 1) {
		return 1;
	}
	auto facta = fact.cache[a];
	auto factb = fact.cache[b];
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

void comp_ppows() {
	for (int i = 2; i <= maxn; i++) {
		auto& xfact = fact.cache[i];
		for (auto [p, e] : xfact) {
			e = d - e + 1;
			int cur = 1;
			for (int i = 0; i < e; i++) {
				if (long(cur) * p > maxn) {
					cur = -1;
					break;
				}
				cur *= p;
			}
			if (cur != -1) {
				cache_ppows[i].push_back(cur);
			}
		}
	}
}

void comp_cnt() {
	for (int i = 2; i <= maxn; i++) {
		auto& xfact = fact.cache[i];
		int& cnt = cache_cnt[i];
		for (auto& [_, e] : xfact) {
			if (e > d) {
				cnt = -1;
				return;
			}
			cnt += e;
		}
	}
}

vector<pair<int, int>> cfact;

template <typename T>
void for_each_factor(const vector<pair<int, int>>& f,
					 const T& t,
					 int ind = 0,
					 int cur = 1) {
	if (ind == sz(f)) {
		t(cur, cfact);
	} else {
		for_each_factor(f, t, ind + 1, cur);
		auto& [p, e] = f[ind];
		cfact.emplace_back(p, 0);
		for (int i = 1; i <= e; i++) {
			cur *= p;
			cfact.back().second = i;
			for_each_factor(f, t, ind + 1, cur);
		}
		cfact.pop_back();
	}
}

struct Counter {
	int odd, even;

	int compute(int x) const {
		int val = -odd + even;
		if (x % 2 == 1) {
			return -val;
		}
		return val;
	}

	friend Counter operator+=(Counter& a, const Counter& b) {
		a.odd += b.odd;
		a.even += b.even;
		return a;
	}
	friend Counter operator+(Counter a, const Counter& b) { return a += b; }

	friend Counter operator-=(Counter& a, const Counter& b) {
		a.odd -= b.odd;
		a.even -= b.even;
		return a;
	}
	friend Counter operator-(Counter a, const Counter& b) { return a += b; }
};

struct SDS {
	int ans = 0;
	vector<int> vals;

	void update(int x) {
		vals.push_back(x);
		for (auto& a : vals) {
			ans += f(a, x);
		}
	}

	int compute() const { return ans; }

	void merge_with(const SDS& x) {
		ans += x.ans;
		for (auto& a : vals) {
			for (auto& b : x.vals) {
				ans += f(a, b);
			}
		}
		vals.insert(vals.end(), begin(x.vals), end(x.vals));
	}
};

struct DS {
	int ans = 0;
	Counter acnt {};
	vector<int> vals {};
	unordered_map<int, Counter> cnts;

	template <bool b = true>
	void update(int x) {
		int cnt = cache_cnt[x];
		if (cnt == -1) {
			return;
		}
		vals.push_back(x);
		if (b) {
			if (cnt % 2 == 1) {
				acnt.odd++;
			} else {
				acnt.even++;
			}
			for_each_factor(fact.cache[x], [&](int x, const auto& _) -> void {
				if (cnt % 2 == 1) {
					cnts[x].odd++;
				} else {
					cnts[x].even++;
				}
			});
		}
		auto& ppows = cache_ppows[x];
		int n = sz(ppows);
		for (int i = 0; i < (1 << n); i++) {
			int cur = 1;
			for (int j = 0; j < n; j++) {
				if ((i >> j) & 1) {
					if (long(cur) * ppows[j] > maxn) {
						cur = -1;
						break;
					}
					cur *= ppows[j];
				}
			}
			if (cur == -1) {
				continue;
			}
			auto it = cnts.find(cur);
			if (it != cnts.end()) {
				if (__builtin_popcount(i) % 2 == 1) {
					ans -= it->second.compute(cnt);
				} else {
					ans += it->second.compute(cnt);
				}
			}
		}
	}

	void merge_with(const DS& ds) {
		for (auto& a : ds.vals) {
			update<true>(a);
		}
//		ans += ds.ans;
//		acnt += ds.acnt;
//		for (auto& [k, v] : ds.cnts) {
//			if (v.compute(0)) {
//				cnts[k] += v;
//			}
//		}
	}

	void merge_with(const SDS& sds) {
		for (auto& a : sds.vals) {
			update(a);
		}
	}

	int compute() const { return ans; }
};

void pdfs(int u) {
	siz[u] = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
		siz[u] += siz[v];
		if (siz[v] > siz[graph[u][0]]) {
			swap(v, graph[u][0]);
		}
	}
}

SDS sdfs(int u) {
	if (!sz(graph[u])) {
		SDS cur;
		cur.update(arr[u]);
		ans[u] = cur.compute();
		return cur;
	}
	SDS cur = sdfs(graph[u][0]);
	for (int i = 1; i < sz(graph[u]); i++) {
		cur.merge_with(sdfs(graph[u][i]));
	}
	cur.update(arr[u]);
	ans[u] = cur.compute();
	return cur;
}

DS dfs(int u) {
	DS cur;
	for (auto& v : graph[u]) {
		if (siz[v] <= bsize) {
			auto x = sdfs(v);
			cur.merge_with(x);
		} else {
			auto x = dfs(v);
			if (v == graph[u][0]) {
				swap(cur, x);
			}
			cur.merge_with(x);
		}
	}
	cur.update(arr[u]);
	ans[u] = cur.compute();
	return cur;
}

void solve() {
	int n;
	cin >> n >> d;
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
	comp_ppows();
	comp_cnt();
	if (sz(graph[0]) == n - 1) {
		DS ds;
		for (int i = 1; i <= n; i++) {
			ds.update(i);
		}
		cout << ds.compute() << endl;
		for (int i = 1; i < n; i++) {
			cout << f(arr[i], arr[i]) << endl;
		}
		return;
	}
	pdfs(0);
	if (siz[0] <= bsize) {
		sdfs(0);
	} else {
		dfs(0);
	}
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