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

const int maxn = 2e5;

struct Fact {
	vector<int> mdiv;

	explicit Fact(int n) : mdiv(n + 1) {
		for (int i = 2; i <= n; i++) {
			if (!mdiv[i]) {
				mdiv[i] = i;
				if (long(i) * i <= n) {
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

int d, arr[maxn];
long ans[maxn];
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
	long odd, even;

	long compute(int x) const {
		long val = -odd + even;
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

struct DS {
	long ans = 0;
	Counter acnt {};
	vector<int> vals;
	__gnu_pbds::gp_hash_table<int, Counter, custom_hash> cnts {};

	void update(int x) {
		vals.push_back(x);
		auto xfact = fact.factorize(x);
		int cnt = 0;
		for (auto& [p, e] : xfact) {
			if (e > d) {
				return;
			}
			cnt += e;
		}
		if (cnt % 2 == 1) {
			acnt.odd++;
		} else {
			acnt.even++;
		}
		for_each_factor(xfact, [&](int x, const auto& _) -> void {
			if (cnt % 2 == 1) {
				cnts[x].odd++;
			} else {
				cnts[x].even++;
			}
		});
		vector<int> ppows;
		for (auto& [p, e] : xfact) {
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
				ppows.push_back(cur);
			}
		}
		int n = sz(ppows);
		Counter ccnt = acnt;
		for (int i = 1; i < (1 << n); i++) {
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
					ccnt -= it->second;
				} else {
					ccnt += it->second;
				}
			}
		}
		ans += ccnt.compute(cnt);
	}

	void merge_with(const DS& ds) {
		for (auto& a : ds.vals) {
			update(a);
		}
	}

	long compute() const { return ans; }

	static DS small_to_large(DS& a, DS& b) {
		if (sz(a.vals) < sz(b.vals)) {
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
	if (sz(graph[0]) == n - 1) {
		DS ds;
		for (int i = 1; i <= n; i++) {
			ds.update(i);
		}
		cout << ds.compute() << endl;
		for (int i = 1; i < n; i++) {
			cout << (arr[i] == 1) << endl;
		}
		return;
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