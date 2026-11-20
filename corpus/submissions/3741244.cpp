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

int egcd(int a, int b, int &x, int &y) {
	if(a == 0) {
		x = 0;
		y = 1;
		return b;
	}
	int xa, ya;
	int gcd = egcd(b % a, a, xa, ya);
	x = ya - (b / a) * xa;
	y = xa;
	return gcd;
}

int inv(int x, int m) {
	if(!x) {
		return -1;
	}
	int a, b;
	if(egcd(x, m, a, b) != 1) {
		return -1;
	}
	a %= m;
	a += m;
	if(a >= m) {
		a -= m;
	}
	return a;
}

int mdiff(int a, int b, int m) {
	a %= m;
	b %= m;
	a -= b;
	if(a < 0) {
		a += m;
	}
	return a;
}

void solve() {
	int n;
	cin >> n;
	int start = 0, arr[n][3];
	for(int i = 0; i < n; i++) {
		cin >> arr[i][0] >> arr[i][1] >> arr[i][2];
		start = max(start, arr[i][0]--);
	}
	for(int i = start; i <= 1e6; i++) {
		for(int j = 0; j < n; j++) {
			for(int k = 0; k < j; k++) {
				long x = inv(mdiff(arr[j][1], arr[k][1], i), i);
				if(x != -1 && (x * mdiff(arr[k][0], arr[j][0], i)) % i <= min(arr[j][2], arr[k][2])) {
					goto loop;
				}
			}
		}
		cout << i << endl;
		return;
		loop:;
	}
	assert(false);
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}