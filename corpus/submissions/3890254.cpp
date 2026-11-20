#ifndef DEBUG
#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")
#endif

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

const int maxn = 50, maxw = 5, maxb = maxn * maxw, logn = 31;
const long minf = -4e18;

#define MATRIX_BASE array<array<long, m>, n>

template<int n, int m>
struct matrix: MATRIX_BASE {
	constexpr matrix(): MATRIX_BASE() {}

	static constexpr matrix<n, m> nmat() {
		matrix<n, m> ret;
		for(int i = 0; i < n; i++) {
			for(int j = 0; j < m; j++) {
				ret[i][j] = minf;
			}
		}
		return ret;
	}
};

template<int n, int m>
constexpr matrix<n, m> nmat = matrix<n, m>::nmat();

#undef MATRIX_BASE

using mat = matrix<maxb, maxb>;
using vec = matrix<1, maxb>;

mat pows[logn + 1];

template<int a, int b, int c>
constexpr matrix<a, b> operator*(const matrix<a, b> &x, const matrix<b, c> &y) {
	matrix<a, c> ret(nmat<a, c>);
	for(int i = 0; i < a; i++) {
		for(int j = 0; j < b; j++) {
			for(int k = 0; k < c; k++) {
				ret[i][k] = max(ret[i][k], x[i][j] + y[j][k]);
			}
		}
	}
	return ret;
}

constexpr void bpow(vec &x, int exp) {
	for(int i = 0; exp; exp >>= 1, i++) {
		if(exp & 1) {
			x = x * pows[i];
		}
	}
}

void pcomp() {
	for(int i = 0; i < logn; i++) {
		pows[i + 1] = pows[i] * pows[i];
	}
}

void solve() {
	int n, m, t, k;
	cin >> n >> m >> t >> k;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	mat base(nmat<maxb, maxb>);
	for(int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		base[u * maxw][v * maxw + w - 1] = arr[v];
	}
	for(int i = 0; i < n; i++) {
		for(int j = 1; j < maxw; j++) {
			base[i * maxw + j][i * maxw + j - 1] = 0;
		}
	}
	pows[0] = base;
	pcomp();
	array<int, 3> fests[k + 1];
	for(int i = 0; i < k; i++) {
		for(int j = 0; j < 3; j++) {
			cin >> fests[i][j];
		}
		fests[i][1]--;
	}
	fests[k] = {t, 0, 0};
	sort(fests, fests + k);
	vec ans(vec::nmat());
	ans[0][0] = 0;
	int prev = 0;
	for(auto &[ct, u, x]: fests) {
		bpow(ans, ct - prev);
		dbg(ans[0][0], ans[0][1]);
		if(ans[0][u * maxw] >= 0) {
			ans[0][u * maxw] += x;
		}
		prev = ct;
	}
	if(ans[0][0] < 0) {
		cout << -1 << endl;
	}else {
		cout << arr[0] + ans[0][0] << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#else
	freopen("delicacy.in", "r", stdin);
	freopen("delicacy.out", "w", stdout);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}