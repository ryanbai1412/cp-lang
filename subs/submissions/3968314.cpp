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

template <typename T>
T mod_inv_in_range(T a, T m) {
	// assert(0 <= a && a < m);
	T x = a, y = m;
	T vx = 1, vy = 0;
	while (x) {
		T k = y / x;
		y %= x;
		vy -= k * vx;
		std::swap(x, y);
		std::swap(vx, vy);
	}
	assert(y == 1);
	return vy < 0 ? m + vy : vy;
}

template <typename T>
T mod_inv(T a, T m) {
	a %= m;
	a = a < 0 ? a + m : a;
	return mod_inv_in_range(a, m);
}

template <int MOD_>
struct modnum {
	static constexpr int MOD = MOD_;
	static_assert(MOD_ > 0, "MOD must be positive");

	using ll = long;

	int v;

	modnum() : v(0) {}
	modnum(ll v_) : v(int(v_ % MOD)) {
		if (v < 0)
			v += MOD;
	}
	explicit operator int() const { return v; }
	friend std::ostream& operator<<(std::ostream& out, const modnum& n) {
		return out << int(n);
	}
	friend std::istream& operator>>(std::istream& in, modnum& n) {
		ll v_;
		in >> v_;
		n = modnum(v_);
		return in;
	}

	friend bool operator==(const modnum& a, const modnum& b) {
		return a.v == b.v;
	}
	friend bool operator!=(const modnum& a, const modnum& b) {
		return a.v != b.v;
	}

	modnum inv() const {
		modnum res;
		res.v = mod_inv_in_range(v, MOD);
		return res;
	}
	friend modnum inv(const modnum& m) { return m.inv(); }
	modnum neg() const {
		modnum res;
		res.v = v ? MOD - v : 0;
		return res;
	}
	friend modnum neg(const modnum& m) { return m.neg(); }

	modnum operator-() const { return neg(); }
	modnum operator+() const { return modnum(*this); }

	modnum& operator++() {
		v++;
		if (v == MOD)
			v = 0;
		return *this;
	}
	modnum& operator--() {
		if (v == 0)
			v = MOD;
		v--;
		return *this;
	}
	modnum& operator+=(const modnum& o) {
		v -= MOD - o.v;
		v = (v < 0) ? v + MOD : v;
		return *this;
	}
	modnum& operator-=(const modnum& o) {
		v -= o.v;
		v = (v < 0) ? v + MOD : v;
		return *this;
	}
	modnum& operator*=(const modnum& o) {
		v = int(ll(v) * ll(o.v) % MOD);
		return *this;
	}
	modnum& operator/=(const modnum& o) { return *this *= o.inv(); }

	friend modnum operator++(modnum& a, int) {
		modnum r = a;
		++a;
		return r;
	}
	friend modnum operator--(modnum& a, int) {
		modnum r = a;
		--a;
		return r;
	}
	friend modnum operator+(const modnum& a, const modnum& b) {
		return modnum(a) += b;
	}
	friend modnum operator-(const modnum& a, const modnum& b) {
		return modnum(a) -= b;
	}
	friend modnum operator*(const modnum& a, const modnum& b) {
		return modnum(a) *= b;
	}
	friend modnum operator/(const modnum& a, const modnum& b) {
		return modnum(a) /= b;
	}
};

using mint = modnum<998244352>;

const int maxn = 50, maxm = 17;

int siz[maxn];
mint fact[maxn], memo[maxn][maxn][2];
vector<int> graph[maxn];

void pcomp() {
	fact[0] = 1;
	for (int i = 1; i < maxn; i++) {
		fact[i] = fact[i - 1] * i;
	}
}

void pdfs(int u) {
	siz[u] = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
		siz[u] += siz[v];
	}
}

mint dp(int u, int k, int cur) {
	auto& ans = memo[u][k][cur];
	if (ans.v != -1) {
		return ans;
	}

	vector<vector<mint>> cxdp(maxm, vector<mint>(k + 1));
	cxdp[cur][cur] = 1;
	int csum = cur;
	for (auto& v : graph[u]) {
		vector<vector<mint>> nxdp(maxm, vector<mint>(k + 1));
		for (int a = 0; a < maxm; a++) {
			auto& ndp = nxdp[a];
			for (int i = 0; i <= csum; i++) {
				for (int j = 0; i + j <= k && j <= siz[v]; j++) {
					ndp[i + j] += cxdp[a][i] * dp(v, j, 0);
					if (a && j) {
						ndp[i + j] += cxdp[a - 1][i] * dp(v, j, 1);
					}
				}
			}
		}
		csum += siz[v];
		cxdp = nxdp;
	}

	ans = 0;
	for (int i = 0; i < maxm; i++) {
		ans += cxdp[i][k] * fact[i];
	}
	assert(0 <= ans && ans < 998244352);
	return ans;
}

void solve() {
	int k, n;
	cin >> n >> k;
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (auto& a : memo) {
		for (auto& b : a) {
			for (auto& c : b) {
				c.v = -1;
			}
		}
	}
	int root = chrono::steady_clock::now().time_since_epoch().count() % n;
	pdfs(root);
	cout << dp(root, k, 0) << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}