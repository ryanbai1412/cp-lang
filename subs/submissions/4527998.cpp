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

const int mod = 1e4;

/*
struct F {
	int m, b;

	F() : m(0), b(0) {}

	F(int x) : m(0), b(x % mod) {}

	F(int m, int b) : m(m % mod), b(b % mod) {}

	F& operator+=(const F& f) {
		m += f.m;
		b += f.b;
		if (m >= mod) {
			m -= mod;
		}
		if (b >= mod) {
			b -= mod;
		}
		return *this;
	}

	F& operator-=(const F& f) {
		m -= f.m;
		b -= f.b;
		if (m < 0) {
			m += mod;
		}
		if (b < 0) {
			b += mod;
		}
		return *this;
	}

	F& operator*=(int x) {
		m = (m * x) % mod;
		b = (b * x) % mod;
		return *this;
	}

	friend F operator+(F a, const F& b) {
		return a += b;
	}

	friend F operator-(F a, const F& b) {
		return a -= b;
	}

	friend F operator*(F a, int b) {
		return a *= b;
	}

	friend F operator*(int a, F b) {
		return b *= a;
	}

	friend ostream& operator<<(ostream& out, const F& f) {
		return out << "{" << f.m << ", " << f.b << "}";
	}
};
*/

struct F {
	int m, b;
	
	F() : m(0), b(0) {}

	F(int x) : m(0), b(x) {}

	F(int m, int b) : m(m), b(b) {}

	F& operator+=(const F& f) {
		m += f.m;
		b += f.b;
		return *this;
	}

	F& operator-=(const F& f) {
		m -= f.m;
		b -= f.b;
		return *this;
	}

	F& operator*=(int x) {
		m *= x;
		b *= x;
		return *this;
	}

	friend F operator+(F a, const F& b) {
		return a += b;
	}

	friend F operator-(F a, const F& b) {
		return a -= b;
	}

	friend F operator*(F a, int b) {
		return a *= b;
	}

	friend F operator*(int a, F b) {
		return b *= a;
	}

	friend ostream& operator<<(ostream& out, const F& f) {
		return out << "{" << f.m << ", " << f.b << "}";
	}
};

void solve(int m, vector<int> arr) {
	int n = sz(arr);
	assert(n <= 500);
	int nxt[n + 1] {};
	for (int i = 1; i <= n; i++) {
		int j = i - 1;
		while (j) {
			j = nxt[j];
			if (arr[j] == arr[i - 1]) {
				nxt[i] = j + 1;
				break;
			}
		}
		dbg(i, nxt[i]);
	}
	F dp[n + 1];
	dp[0] = {1, 0};
	for (int i = 0; i < n; i++) {
		if (!i) {
			dp[1] = dp[0] - m;
#ifdef DEBUG
			fprintf(stderr, "dp[1] = dp[0] - %d\n", m);
#endif
			continue;
		}
		int j = i;
		while (j) {
			j = nxt[j];
			if (arr[j] == arr[i]) {
				j++;
				break;
			}
		}
		dp[i + 1] = m * (dp[i] - dp[nxt[i]]) + dp[j];
#ifdef DEBUG
		fprintf(stderr, "dp[%d] = %d(dp[%d] - dp[%d]) + dp[%d]\n", i + 1, m, i, nxt[i], j);
#endif
	}
	for (int i = 0; i <= n; i++) {
		dbg(i, dp[i]);
	}
	printf("%04d\n", -dp[n].b / dp[n].m % 10000);
}

void solve() {
	int m, t;
	cin >> m >> t;
	while (t--) {
		int n;
		cin >> n;
		vector<int> arr(n);
		for (auto& a : arr) {
			cin >> a;
		}
		solve(m, arr);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}