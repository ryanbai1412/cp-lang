#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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

const int maxn = 205, maxm = 505, inf = 2e9 + 5, mod = 1e9 + 7;

struct mint {
	static int norm(long x) {
		if (!(-mod <= x && x < mod)) {
			x %= mod;
		}
		if (x < 0) {
			x += mod;
		}
		return int(x);
	}

	int x;

	mint() : x(0) {}
	mint(int x) : x(norm(x)) {}
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

	bool operator==(const mint& m) const {
		return x == m.x;
	}

	bool operator!=(const mint& m) const {
		return x != m.x;
	}

	IMPL_FRIEND_OPS(mint)
};

mint bpow(mint base, long exp) {
	mint ans = 1;
	while (exp) {
		if (exp & 1) {
			ans *= base;
		}
		base *= base;
		exp >>= 1;
	}
	return ans;
}

mint inv(mint x) {
	return bpow(x, mod - 2);
}

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

	void optimize() {
		while (sz(coef) && coef.back() == 0) {
			coef.pop_back();
		}
	}

	bool operator==(const Poly& p) const {
		return coef == p.coef;
	}

	bool operator!=(const Poly& p) const {
		return coef != p.coef;
	}

	static Poly constant(mint x) {
		return {{x}};
	}

	static Poly identity() {
		return linear(1, 0);
	}

	static Poly linear(mint m, mint b) {
		return {{b, m}};
	}
};

Poly pcoef[maxn];

void pcomp() {
	Poly pvals[maxn], xvals[maxn];
	for (int i = 0; i < maxn; i++) {
		pvals[i] = Poly::constant(1);
		for (int j = 0; j < i; j++) {
			pvals[i] *= Poly::linear(1, -j);
		}
		mint cur = 0;
		for (int j = 0; j <= i; j++) {
			cur *= inf - j;
		}
		xvals[i] =
			(Poly::linear(1, 1) * pvals[i] + Poly::constant(cur)) * Poly::constant(inv(i + 1));
	}
	for (int i = 0; i < maxn; i++) {
		Poly p {vector<mint>(i + 1)}, q {vector<mint>(i + 1)};
		p.coef[i] = 1;
		for (int j = i; j >= 0; j--) {
			mint cur = p.coef[j];
			q.coef[j] = cur;
			p -= Poly::constant(cur) * pvals[j];
		}
		for (int j = 0; j <= i; j++) {
			pcoef[i] += xvals[j] * Poly::constant(q.coef[j]);
		}
	}
}

Poly psumof(const Poly& p) {
	auto& c = p.coef;
	int n = sz(c);
	vector<mint> q(n + 1);
	for (int i = 0; i < n; i++) {
		for (int j = 0; j <= i + 1; j++) {
			q[j] += c[i] * pcoef[i].coef[j];
		}
	}
	return {q};
}

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

	mint sumall() const {
		mint ans = 0;
		for (int i = 0; i < sz(polys) - 1; i++) {
			auto p = psumof(polys[i].second);
			ans += p(polys[i + 1].first - 1) - p(polys[i].first - 1);
		}
		return ans;
	}

	void optimize() {
		for (auto& [_, x] : polys) {
			x.optimize();
		}
		vector<pair<int, Poly>> nxt;
		for (int i = 0; i < sz(polys); i++) {
			if (!i || polys[i - 1].second != polys[i].second) {
				nxt.push_back(polys[i]);
			}
		}
		swap(polys, nxt);
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
bool vis[maxn];
int k, start, siz[maxn], arr[maxn][2], merges = 0;
vector<int> graph[maxn];

Piecewise cnt(const Piecewise& l, const Piecewise& r) {
	return (r * (r + Poly::constant(1)) - l * (l - Poly::constant(1))) * Poly::constant(inv(2));
}

struct Answer {
	array<array<Piecewise, 2>, 2> dp;

	Answer& operator+=(const Answer& ans) {
		for (int i = 0; i < 2; i++) {
			for (int j = 0; j < 2; j++) {
				dp[i][j] += ans.dp[i][j];
			}
		}
		return *this;
	}

	friend Answer operator+(Answer a, const Answer& b) {
		return a += b;
	}

	friend Answer operator*(const Answer& ansa, const Answer& ansb) {
		merges++;
		auto& a = ansa.dp;
		auto& b = ansb.dp;
		array<array<Piecewise, 2>, 2> ndp;
		for (auto& x : ndp) {
			for (auto& y : x) {
				y = Piecewise::constant(0);
			}
		}
		for (int i = 0; i < 2; i++) {
			for (int j = 0; j < 2; j++) {
				ndp[i || j][0] += a[i][0] * b[j][0];
				ndp[i || j][1] += a[i][0] * b[j][1] + a[i][1] * b[j][0];
			}
		}
		return {move(ndp)};
	}

	void updg() const {
		ans1 += dp[1][0].sumall();
		ans2 += dp[1][1].sumall();
	}

	void optimize() {
		for (auto& a : dp) {
			for (auto& b : a) {
				b.optimize();
			}
		}
	}

	static Answer def() {
		return {{array<Piecewise, 2> {Piecewise::constant(1), Piecewise::constant(0)},
				 {Piecewise::constant(0), Piecewise::constant(0)}}};
	}
} answers[maxn], adef = Answer::def();

void pdfs(int u, int p = -1) {
	siz[u] = 1;
	for (auto& v : graph[u]) {
		if (v != p) {
			pdfs(v, u);
			siz[u] += siz[v];
		}
	}
}

vector<Answer> cur;

void xdfs(int u, int p, const Answer& ans) {
	Answer nans = ans * answers[u];
	cur.push_back(nans);
	for (auto& v : graph[u]) {
		if (v != p && !vis[v]) {
			xdfs(v, u, nans);
		}
	}
}

void dfs(int u) {
	int targ = siz[u] / 2;
	while (true) {
		pair<int, int> opt {-1, -1};
		for (auto& v : graph[u]) {
			if (!vis[v]) {
				opt = max(opt, pair<int, int> {siz[v], v});
			}
		}
		auto& [x, v] = opt;
		if (x > targ) {
			siz[u] -= siz[v];
			siz[v] += siz[u];
			u = v;
		} else {
			break;
		}
	}
	Answer cans = adef, tot = adef;
	vis[u] = true;
	for (auto& v : graph[u]) {
		if (!vis[v]) {
			cur.clear();
			xdfs(v, u, adef);
			for (auto& a : cur) {
				tot += cans * a;
			}
			for (auto& a : cur) {
				cans += a;
			}
			cans.optimize();
		}
	}
	tot.optimize();
	(tot * answers[u]).updg();
	for (auto& v : graph[u]) {
		if (!vis[v]) {
			dfs(v);
		}
	}
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
		answers[i].dp = {
			array<Piecewise, 2> {
				Piecewise::from_range(
					l - k - 1, r,
					Piecewise::fn_min(r - k) + Poly::constant(k) - Piecewise::fn_max(l - 1),
					Poly::constant(0)),
				Piecewise::from_range(l - k, r - 1,
									  cnt(Piecewise::fn_max(l - 1) + Poly::constant(1),
										  Piecewise::fn_min(r - k) + Poly::constant(k)),
									  Poly::constant(0))},
			{Piecewise::from_range(l, r, Poly::constant(1), Poly::constant(0)),
			 Piecewise::from_range(l, r, Poly::constant(1), Poly::constant(0)) * Poly::identity()}};
	}
	pdfs(0);
	dfs(0);
	cout << ans1 << endl;
	cout << ans2 << endl;
	//	cout << merges << endl;
}

int main() {
	//	freopen("tree.in", "r", stdin);
	//	freopen("tree.out", "w", stdout);
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	solve();
}