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

#define IMPL_FRIEND_OPS(CLS)                                     \
	friend CLS operator+(CLS a, const CLS& b) { return a += b; } \
	friend CLS operator-(CLS a, const CLS& b) { return a -= b; } \
	friend CLS operator*(CLS a, const CLS& b) { return a *= b; }

#define IMPL_FRIEND_OPS2(CLS1, CLS2)                                \
	friend CLS1 operator+(CLS1 a, const CLS2& b) { return a += b; } \
	friend CLS1 operator+(const CLS2& b, CLS1 a) { return a += b; } \
	friend CLS1 operator-(CLS1 a, const CLS2& b) { return a -= b; } \
	friend CLS1 operator-(const CLS2& b, CLS1 a) { return a -= b; } \
	friend CLS1 operator*(CLS1 a, const CLS2& b) { return a *= b; } \
	friend CLS1 operator*(const CLS2& b, CLS1 a) { return a *= b; }

const int maxn = 205, maxm = 505, inf = 2e9 + 5;

struct mint {
	static constexpr int mod = 1e9 + 7;

	static int norm(long x) {
		x %= mod;
		if (x < 0) {
			x += mod;
		}
		return int(x);
	}

	int x;

	mint() : x(0) {}
	mint(int x) : x(long(x)) {}
	mint(long x) : x(norm(x)) {}

	friend ostream& operator<<(ostream& out, const mint& m) {
		return out << m.x;
	}

	mint& operator+=(const mint& m) {
		x += m.x;
		if (x >= mod) {
			x -= mod;
		}
		return *this;
	}

	mint& operator-=(const mint& m) {
		x -= m.x;
		if (x < 0) {
			x += mod;
		}
		return *this;
	}

	mint& operator*=(const mint& m) {
		x = int((long(x) * m.x) % mod);
		return *this;
	}

	IMPL_FRIEND_OPS(mint)
};

struct Poly {
	vector<mint> coef;

	friend ostream& operator<<(ostream& out, const Poly& p) {
		bool prev = false;
		for (int i = 0; i < sz(p.coef); i++) {
			if (p.coef[i].x == 0) {
				continue;
			}
			if (prev) {
				cout << " + ";
			}
			prev = true;
			if (!i) {
				cout << p.coef[i];
			} else if (i == 1) {
				cout << p.coef[i] << "x";
			} else {
				cout << p.coef[i] << "x^" << i;
			}
		}
		if (!prev) {
			cout << "0";
		}
		return out;
	}

	template <typename T>
	void merge(const Poly& p, const T& t) {
		if (sz(coef) < sz(p.coef)) {
			coef.resize(sz(p.coef));
		}
		for (int i = 0; i < sz(p.coef); i++) {
			t(coef[i], p.coef[i]);
		}
	}

	Poly& operator+=(const Poly& p) {
		merge(p, [&](mint& a, const mint& b) -> void {
			a += b;
		});
		return *this;
	}

	Poly& operator-=(const Poly& p) {
		merge(p, [&](mint& a, const mint& b) -> void {
			a -= b;
		});
		return *this;
	}

	Poly& operator*=(const Poly& p) {
		int n = sz(coef), m = sz(p.coef);
		vector<mint> ans(n + m - 1);
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < m; j++) {
				ans[i + j] += coef[i] * p.coef[j];
			}
		}
		swap(coef, ans);
		return *this;
	}

	IMPL_FRIEND_OPS(Poly)

	mint operator()(mint x) const {
		mint base = 1, ans = 0;
		for (auto& a : coef) {
			ans += a * base;
			base *= x;
		}
		return ans;
	}

	static Poly constant(mint x) {
		return {{x}};
	}

	static Poly identity() {
		return {{0, 1}};
	}
};

struct Piecewise {
	vector<pair<int, Poly>> polys;

	friend ostream& operator<<(ostream& out, const Piecewise& p) {
		cout << "{";
		for (int i = 0; i < sz(p.polys); i++) {
			if (i) {
				cout << ", ";
			}
			auto& [a, b] = p.polys[i];
			cout << "{" << a << ", " << b << "}";
		}
		cout << "}";
		return out;
	}

	Piecewise& operator+=(const Poly& p) {
		for (auto& [_, x] : polys) {
			x += p;
		}
		return *this;
	}

	Piecewise& operator-=(const Poly& p) {
		for (auto& [_, x] : polys) {
			x -= p;
		}
		return *this;
	}

	Piecewise& operator*=(const Poly& p) {
		for (auto& [_, x] : polys) {
			x *= p;
		}
		return *this;
	}

	IMPL_FRIEND_OPS2(Piecewise, Poly)

	template <typename T>
	void merge(const Piecewise& p, const T& t) {
		vector<pair<int, Poly>> ans;
		int n = sz(polys), m = sz(p.polys), i = 1, j = 1;
		assert(polys[0].first == -inf && p.polys[0].first == -inf);
		ans.emplace_back(-inf, t(polys[0].second, p.polys[0].second));
		while (i < n || j < m) {
			int ci = i == n ? inf : polys[i].first;
			int cj = j == m ? inf : p.polys[j].first;
			if (ci < cj) {
				ans.emplace_back(ci, t(polys[i].second, p.polys[j - 1].second));
				i++;
			} else if (cj < ci) {
				ans.emplace_back(cj, t(polys[i - 1].second, p.polys[j].second));
				j++;
			} else {
				ans.emplace_back(ci, t(polys[i].second, p.polys[j].second));
				i++;
				j++;
			}
		}
		swap(polys, ans);
	}

	Piecewise& operator+=(const Piecewise& p) {
		merge(p, [&](const Poly& a, const Poly& b) -> Poly {
			return a + b;
		});
		return *this;
	}

	Piecewise& operator-=(const Piecewise& p) {
		merge(p, [&](const Poly& a, const Poly& b) -> Poly {
			return a - b;
		});
		return *this;
	}

	Piecewise& operator*=(const Piecewise& p) {
		merge(p, [&](const Poly& a, const Poly& b) -> Poly {
			return a * b;
		});
		return *this;
	}

	IMPL_FRIEND_OPS(Piecewise)

	mint operator()(int x) const {
		int l = 0, r = sz(polys);
		while (l < r) {
			int mid = (l + r) / 2;
			if (polys[mid].first > x) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return polys[l - 1].second(x);
	}

	static Piecewise constant(mint x) {
		return {{{-inf, Poly::constant(x)}}};
	}

	static Piecewise from_range(int l, int r, const Poly& p, const Poly& def) {
		return {{{-inf, def}, {l, p}, {r + 1, def}}};
	}

	static Piecewise from_range(int l, int r, const Piecewise& p, const Poly& def) {
		vector<pair<int, Poly>> ans;
		ans.emplace_back(-inf, def);
		int n = sz(p.polys);
		int i;
		for (i = 0; i < n && p.polys[i].first <= l; i++)
			;
		ans.emplace_back(l, p.polys[i - 1].second);
		int start = i;
		for (i = 0; i < n && p.polys[i].first <= r; i++)
			;
		ans.insert(ans.end(), p.polys.begin() + start, p.polys.begin() + i);
		ans.emplace_back(r + 1, def);
		return {ans};
	}

	static Piecewise fn_min(int x) {
		return {{{-inf, Poly::identity()}, {x, Poly::constant(x)}}};
	}

	static Piecewise fn_max(int x) {
		return {{{-inf, Poly::constant(x)}, {x, Poly::identity()}}};
	}
};

mint ans1, ans2;
Piecewise dpc[maxn][2][2];
int k, arr[maxn][2];
vector<int> st, graph[maxn];

mint cnt(long l, long r) {
	if (l > r) {
		return 0;
	}
	return r * (r + 1) / 2 - l * (l - 1) / 2;
}

void csolve() {
	Piecewise dp[2] {Piecewise::constant(1), Piecewise::constant(0)};
	for (auto& ind : st) {
		Piecewise a = dp[0] * dpc[ind][0][0];
		Piecewise b = dp[0] * dpc[ind][0][1] + dp[1] * dpc[ind][1][1];
		dp[0] = a;
		dp[1] = b;
	}
	for (int i = 0; i < maxm; i++) {
		ans1 += dp[1](i);
	}
	//	mint dp[2][maxm] {}, dp2[2][maxm] {};
	//	fill(begin(dp[0]), end(dp[0]), 1);
	//	for (auto& [l, r] : arr) {
	//		for (int i = 0; i < maxm; i++) {
	//			{
	//				mint a = dp2[0][i] * max(0, min(r, i + k) - max(l, i + 1) + 1) +
	//						 dp[0][i] * cnt(max(l, i + 1), min(r, i + k));
	//				mint b = dp2[1][i] * max(0, min(r, i + k) - max(l, i) + 1) +
	//						 dp[1][i] * cnt(max(l, i), min(r, i + k));
	//				if (l <= i && i <= r) {
	//					b += dp2[0][i] + dp[0][i] * i;
	//				}
	//				dp2[0][i] = a;
	//				dp2[1][i] = b;
	//			}
	//			{
	//				mint a = dp[0][i] * max(0, min(r, i + k) - max(l, i + 1) + 1);
	//				mint b = dp[1][i] * max(0, min(r, i + k) - max(l, i) + 1);
	//				if (l <= i && i <= r) {
	//					b += dp[0][i];
	//				}
	//				dp[0][i] = a;
	//				dp[1][i] = b;
	//			}
	//		}
	//	}
	//	ans1 += accumulate(begin(dp[1]), end(dp[1]), mint(0));
	//	ans2 += accumulate(begin(dp2[1]), end(dp2[1]), mint(0));
}

void dfs(int u, int p = -1) {
	st.push_back(u);
	if (st[0] <= u) {
		csolve();
	}
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	st.pop_back();
}

void solve() {
	int n;
	cin >> n >> k;
	for (int i = 0; i < n; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		auto& [l, r] = arr[i];
		dpc[i][0][0] = Piecewise::from_range(
			l - k - 1, r, Piecewise::fn_min(r - k) + Poly::constant(k) - Piecewise::fn_max(l - 1),
			Poly::constant(0));
		dpc[i][1][1] = Piecewise::from_range(
			l - k - 1, r + 1,
			Piecewise::fn_min(r - k) + Poly::constant(k) - Piecewise::fn_max(l) + Poly::constant(1),
			Poly::constant(0));
		dpc[i][0][1] = Piecewise::from_range(l, r, Poly::constant(1), Poly::constant(0));
	}
//	for (int i = -100; i < 100; i++) {
//		auto& [l, r] = arr[0];
//		cerr << dpc[0][0][0](i) << " " << max(0, min(r, i + k) - max(l, i + 1) + 1) << endl;
//		cerr << dpc[0][0][1](i) << " " << (l <= i && i <= r) << endl;
//		cerr << dpc[0][1][1](i) << " " << max(0, min(r, i + k) - max(l, i) + 1) << endl;
//	}
//	dbg(dpc[0][0][0]);
	for (int i = 0; i < n; i++) {
		dfs(i);
	}
	cout << ans1 << endl;
	cout << ans2 << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}