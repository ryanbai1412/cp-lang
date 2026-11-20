#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

constexpr int mod = 1e9 + 7, maxn = 310, iinf = 1e9 + 10;

template<typename T>
T bpow(T base, T exp) {
	long ans = 1;
	while(exp) {
		if(exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

struct mint {
	static int norm(long x) {
		x %= mod;
		if(x < 0) {
			x += mod;
		}
		return x;
	}

	static long inv(long x) {
		return bpow(x, long(mod - 2));
	}

	int x;

	mint(): x(0) {}

	mint(int x): x(norm(x)) {}

	mint(long x): x(norm(x)) {}

	friend istream &operator>>(istream &in, mint &m) {
		long x;
		in >> x;
		m = mint(x);
		return in;
	}

	friend ostream &operator<<(ostream &out, const mint &m) {
		return out << m.x;
	}

	mint operator+=(const mint &m) {
		x += m.x;
		if(x >= mod) {
			x -= mod;
		}
		return *this;
	}

	mint operator-=(const mint &m) {
		x -= m.x;
		if(x < 0) {
			x += mod;
		}
		return *this;
	}

	mint operator*=(const mint &m) {
		x = (long(x) * m.x) % mod;
		return *this;
	}

	mint operator/=(const mint &m) {
		x = (long(x) * inv(m.x)) % mod;
		return *this;
	}

	mint operator+() {
		return *this;
	}

	mint operator-() {
		return 0 - *this;
	}

	bool operator==(const mint &m) {
		return x == m.x;
	}

	friend mint operator+(mint a, const mint &b) {
		return a += b;
	}

	friend mint operator-(mint a, const mint &b) {
		return a -= b;
	}

	friend mint operator*(mint a, const mint &b) {
		return a *= b;
	}

	friend mint operator/(mint a, const mint &b) {
		return a /= b;
	}
};

template<typename T>
struct mat: vector<vector<T>> {
	using vector<vector<T>>::vector;

	mat(int n, int m): vector<vector<T>>(n, vector<T>(m)) {}
};

mint fact[maxn + 1];
mat<mint> shleftMat(maxn, maxn), altformMat(maxn, maxn), psumMat(maxn, maxn + 1);

struct Poly: vector<mint> {
	using vector<mint>::vector;

	friend Poly operator*(const Poly &x, const mat<mint> &y) {
		int n = sz(x);
		Poly ans(n);

		for(int i = 0; i < n; i++) {
			for(int j = 0; j < n; j++) {
				ans[j] += x[i] * y[i][j];
			}
		}

		return ans;
	}

	Poly psum() const {
		auto next = *this;
		next.push_back(0);
		return next * psumMat;
	}

	Poly shleft() const {
		return *this * shleftMat;
	}

	friend Poly operator+=(Poly &a, const Poly &b) {
		int n = sz(a), m = sz(b);
		a.resize(max(n, m));

		for(int i = 0; i < m; i++) {
			a[i] += b[i];
		}

		return a;
	}

	friend Poly operator-=(Poly &a, const Poly &b) {
		return a += -b;
	}

	friend Poly operator*=(Poly &a, const Poly &b) {
		int n = sz(a), m = sz(b);
		Poly ans(n + m - 1);

		for(int i = 0; i < n; i++) {
			for(int j = 0; j < m; j++) {
				ans[i + j] += a[i] * b[j];
			}
		}

		a = ans;
		return a;
	}

	friend Poly operator*=(Poly &a, mint x) {
		for(auto &b: a) {
			b *= x;
		}
		return a;
	}

	friend Poly operator+(Poly a, const Poly &b) {
		return a += b;
	}

	friend Poly operator-(Poly a, const Poly &b) {
		return a -= b;
	}

	friend Poly operator*(Poly a, const Poly &b) {
		return a *= b;
	}

	friend Poly operator*(Poly a, mint x) {
		return a *= x;
	}

	friend Poly operator-(const Poly &a) {
		Poly ans {a};

		for(auto &a: ans) {
			a = -a;
		}

		return ans;
	}

	mint operator()(mint x) const {
		const Poly &p = *this;

		mint ans = 0, xpow = 1;
		for(int i = 0; i < sz(p); i++) {
			ans += p[i] * xpow;
			xpow *= x;
		}

		return ans;
	}
};

struct LPoly {
	int l, r;
	Poly p;

	LPoly psum() const {
		return {l, r, p.psum()};
	}

	LPoly shleft() const {
		return {l + 1, r + 1, p.shleft()};
	}
};

struct Func: vector<LPoly> {
	using vector<LPoly>::vector;

	Func psum() const {
		Func pols = *this;

		mint add = 0;
		int last = 0;
		for(auto &a: pols) {
			a = a.psum();
			if(!sz(a.p)) {
				a.p = {0};
			}
			a.p[0] += add - a.p(a.l - 1);
			add = a.p(a.r - 1);
			last = a.r;
		}

		pols.push_back({last, iinf, {add}});

		return pols;
	}

	Func shleft() const {
		Func pols = *this;
		for(auto &a: pols) {
			a = a.shleft();
		}
		return pols;
	}

	Func limit(int l, int r) const {
		Func ans {};

		for(auto[lx, rx, p]: *this) {
			lx = max(lx, l);
			rx = min(rx, r);
			if(lx < rx) {
				ans.push_back({lx, rx, p});
			}
		}

		return ans;
	}

	mint operator()(int x) const {
		for(auto &[lx, rx, p]: *this) {
			if(x < rx) {
				if(lx <= x) {
					return p(x);
				}
				return 0;
			}
		}
		return 0;
	}
};

LPoly operator*(const LPoly &a, const LPoly &b) {
	if(a.l > b.l) {
		return b * a;
	}
	assert(b.l < a.r);
	if(a.r < b.r) {
		return {b.l, a.r, a.p * b.p};
	}
	return {b.l, b.r, a.p * b.p};
}

Func operator*(const Func &a, const Func &b) {
	Func ans {};

	int i = 0, j = 0;
	while(i < sz(a) && j < sz(b)) {
		if(a[i].l == b[j].l) {
			ans.push_back(a[i] * b[j]);
		}else if(a[i].l < b[j].l) {
			if(a[i].r <= b[j].l) {
				i++;
				continue;
			}
			ans.push_back(a[i] * b[j]);
		}else {
			if(b[j].r <= a[i].l) {
				j++;
				continue;
			}
			ans.push_back(a[i] * b[j]);
		}
		if(a[i].r == b[j].r) {
			i++;
			j++;
		}else if(a[i].r < b[j].r) {
			i++;
		}else {
			j++;
		}
	}

	return ans;
}

Func operator+(const Func &xa, const Func &xb) {
	Func a {}, b {};
	int prev = 0;
	for(auto &[l, r, p]: xa) {
		if(prev < l) {
			a.push_back({prev, l, {0}});
		}
		a.push_back({l, r, p});
		prev = r;
	}
	if(prev < iinf) {
		a.push_back({prev, iinf, {0}});
	}
	prev = 0;
	for(auto &[l, r, p]: xb) {
		if(prev < l) {
			b.push_back({prev, l, {0}});
		}
		b.push_back({l, r, p});
		prev = r;
	}
	if(prev < iinf) {
		b.push_back({prev, iinf, {0}});
	}

	Func ans {};

	for(int i = 0, j = 0, k = 0; k < iinf;) {
		if(a[i].r == k) {
			i++;
		}
		if(b[j].r == k) {
			j++;
		}
		int next = min(a[i].r, b[j].r);
		ans.push_back({k, next, a[i].p + b[j].p});
		k = next;
	}

	return ans;
}

mint choose(int n, int k) {
	if(n < 0 || k < 0 || n - k < 0) {
		return 0;
	}

	return fact[n] / (fact[k] * fact[n - k]);
}

vector<mint> calcPowSum(int n) {
	// current polynomial in standard form
	vector<mint> f = altformMat[n];

	// current polynomial in alternative form
	vector<mint> g(n + 1);
	g[n] = 1;

	for(int i = n - 1; i >= 0; i--) {
		g[i] = -f[i];
		for(int j = 0; j <= i; j++) {
			f[j] += g[i] * altformMat[i][j];
		}
	}

	Poly ans(n + 1);
	Poly mul {{1, 1}};
	for(int i = 0; i <= n; i++) {
		ans += mul * (g[i] / (i + 1));

		mul *= Poly {{-i, 1}};
	}

	return ans;
}

void pcomp() {
	fact[0] = 1;
	for(int i = 0; i < maxn; i++) {
		fact[i + 1] = fact[i] * (i + 1);
	}

	for(int i = 0; i < maxn; i++) {
		for(int j = 0; j <= i; j++) {
			shleftMat[i][j] = choose(i, j);
			if((i ^ j) & 1) {
				shleftMat[i][j] = -shleftMat[i][j];
			}
		}
	}

	Poly cur {{1}};
	for(int i = 0; i < maxn; i++) {
		altformMat[i] = cur;
		altformMat[i].resize(maxn);
		cur *= Poly {{-i, 1}};
	}

	for(int i = 0; i < maxn; i++) {
		// f(x) = sum of k^i for k from 0 to x
		vector<mint> f = calcPowSum(i);

		f.resize(maxn + 1);
		psumMat[i] = f;
	}
}

int n, arr[maxn][2];
bool vis[maxn + 1][maxn + 1];
Func memo[maxn + 1][maxn + 1];

Func dp(int l, int r) {
	if(l > r) {
		return {{0, iinf, {1}}};
	}else if(l == r) {
		return {{0,         arr[l][0], {0}},
				{arr[l][0], arr[l][1], {-arr[l][0] + 1, 1}},
				{arr[l][1], iinf,      {arr[l][1] - arr[l][0]}}};
	}else if(vis[l][r]) {
		return memo[l][r];
	}

	Func &cur = memo[l][r];
	cur = {};
	auto process = [&](int i) -> void {
		cur = cur + (dp(l, i - 1) * dp(i + 1, r).shleft()).limit(arr[i][0], arr[i][1]);
	};

	int mid = (l + r) / 2;
	process(mid);
	process(mid + 1);
	if((r - l + 1) & 1) {
		process(mid - 1);
	}

	cur = cur.psum();

	vis[l][r] = true;
	return cur;
}

void solve() {
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i][0] >> arr[i][1];
		arr[i][1]++;
	}
	Func f = dp(0, n - 1);
	if(f.empty()) {
		cout << 0 << endl;
		return;
	}
	cout << f(iinf - 1) << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	pcomp();
	Func a {{1, 3, {0, 1}}};
	Func b {{1, 2, {0, 1}}};
	Func c = a + b;
	dbg(c(0), c(1), c(2), c(3), c(4), c.psum()(5));
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}