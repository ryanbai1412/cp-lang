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

struct cint {
	long real, omega;

	constexpr cint() : cint(0) {}
	constexpr cint(int x) : cint(long(x)) {}
	constexpr cint(long x) : cint(x, 0) {}
	constexpr cint(long _real, long _omega) : real(_real), omega(_omega) {}
	
	constexpr bool operator==(const cint &x) const {
		return real == x.real && omega == x.omega;
	}

	constexpr cint& operator+=(const cint& x) {
		real += x.real;
		omega += x.omega;
		return *this;
	}

	constexpr cint& operator-=(const cint& x) {
		real -= x.real;
		omega -= x.omega;
		return *this;
	}

	constexpr cint& operator*=(const cint& x) {
		long nreal = real * x.real - omega * x.omega;
		long nomega = real * x.omega + omega * x.real - omega * x.omega;
		real = nreal;
		omega = nomega;
		return *this;
	}

	constexpr cint& operator/=(const cint& x) {
		if (x.omega == 0) {
			real /= x.real;
			omega /= x.real;
		} else {
			long nreal = (2 * omega - real) / 3;
			long nomega = (omega - 2 * real) / 3;
			real = nreal;
			omega = nomega;
		}
		return *this;
	}

	constexpr friend cint operator+(cint a, const cint& b) {
		return a += b;
	}

	constexpr friend cint operator-(cint a, const cint& b) {
		return a -= b;
	}

	constexpr friend cint operator*(cint a, const cint& b) {
		return a *= b;
	}

	constexpr friend cint operator/(cint a, const cint& b) {
		return a /= b;
	}
};

constexpr cint omega(0, 1), omegasq = omega * omega, osqsm = omega + omegasq,
							osqdi = omega - omegasq, const1 = 2 - osqsm,
							const2 = 1 - omegasq;

vector<cint> conv(const vector<cint>& x) {
	if (sz(x) == 1) {
		return {x[0] * x[0]};
	}
	int n = sz(x), nn = n / 3;
	vector<cint> a(nn);
	for (int i = 0; i < nn; i++) {
		a[i] = x[i] + x[i + nn] + x[i + nn + nn];
	}
	a = conv(a);
	vector<cint> b(nn);
	for (int i = 0; i < nn; i++) {
		b[i] = x[i] + x[i + nn] * omega + x[i + nn + nn] * omegasq;
	}
	b = conv(b);
	vector<cint> c(nn);
	for (int i = 0; i < nn; i++) {
		c[i] = x[i] + x[i + nn] * omegasq + x[i + nn + nn] * omega;
	}
	c = conv(c);
	vector<cint> ans(n);
	for (int i = 0; i < nn; i++) {
		cint e = ((b[i] + c[i]) - osqsm * a[i]) / const1;
		cint f = (b[i] - omegasq * a[i] - const2 * e) / osqdi;
		cint g = (c[i] - omegasq * a[i] - const2 * e) / osqdi;
		ans[i] = e;
		ans[i + nn] = f;
		ans[i + nn + nn] = g;
	}
	return ans;
}

void solve() {
	int n, k;
	cin >> n >> k;
	int m = 1;
	for (int i = 0; i < k; i++) {
		m *= 3;
	}
	vector<int> arr(n);
	for (auto& a : arr) {
		int cur = 0;
		string s;
		cin >> s;
		for (auto& c : s) {
			cur = cur * 3 + (c - '1');
		}
		a = cur;
	}
	vector<cint> v(m);
	for (auto& a : arr) {
		v[a] += 1;
	}
	auto convd = conv(v);
	long ans = 0;
	for (auto& a : arr) {
		int cur = 0, cpow = 1;
		while (a) {
			cur += (3 - a % 3) % 3 * cpow;
			cpow *= 3;
			a /= 3;
		}
		ans += convd[cur].real - 1;
	}
	cout << ans / 6 << endl;
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