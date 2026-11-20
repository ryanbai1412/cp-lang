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

vector<int> sent;

#ifdef LOCAL

void send(int x) {
	sent.push_back(x);
}

void output(int x) {
	cout << x << endl;
}

#endif

vector<vector<int>> ways;

void pcomp(int n = 10, vector<int> cur = {}) {
	if(n) {
		int last = sz(cur) ? cur.back() : 0;
		for(int i = last; i < 4; i++) {
			cur.push_back(i);
			pcomp(n - 1, cur);
			cur.pop_back();
		}
	}else {
		ways.push_back(cur);
	}
}

void encode(int n, int arr[]) {
	if(ways.empty()) {
		pcomp();
	}
	assert(sz(ways) == 286);
	for(int i = 0; i < n; i++) {
		for(auto &a: ways[arr[i]]) {
			send((i << 2) | a);
		}
	}
}

void decode(int n, int m, int arr[]) {
	vector<int> message[n];
	for(int i = 0; i < m; i++) {
		message[arr[i] >> 2].push_back(arr[i] & 3);
	}
	for(int i = 0; i < n; i++) {
		sort(begin(message[i]), end(message[i]));
		auto it = find(begin(ways), end(ways), message[i]);
		output(it - ways.begin());
	}
}

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	encode(n, arr);
	int m = sz(sent);
	cout << "USED " << m << endl;
	int encoded[m];
	for(int i = 0; i < m; i++) {
		encoded[i] = sent[i];
	}
	shuffle(encoded, encoded + m, mt19937(chrono::steady_clock::now().time_since_epoch().count()));
	decode(n, m, encoded);
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

vector<int> sent;

#ifdef LOCAL

void send(int x) {
	sent.push_back(x);
}

void output(int x) {
	cout << x << endl;
}

#endif

vector<vector<int>> ways;

void pcomp(int n = 10, vector<int> cur = {}) {
	if(n) {
		int last = sz(cur) ? cur.back() : 0;
		for(int i = last; i < 4; i++) {
			cur.push_back(i);
			pcomp(n - 1, cur);
			cur.pop_back();
		}
	}else {
		ways.push_back(cur);
	}
}

void encode(int n, int arr[]) {
	if(ways.empty()) {
		pcomp();
	}
	for(int i = 0; i < n; i++) {
		for(auto &a: ways[arr[i]]) {
			send((i << 2) | a);
		}
	}
}

void decode(int n, int m, int arr[]) {
	vector<int> message[n];
	for(int i = 0; i < m; i++) {
		message[arr[i] >> 2].push_back(arr[i] & 3);
	}
	for(int i = 0; i < n; i++) {
		sort(begin(message[i]), end(message[i]));
		auto it = find(begin(ways), end(ways), message[i]);
		assert(it != ways.end());
		output(it - ways.begin());
	}
}

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	encode(n, arr);
	int m = sz(sent);
	cout << "USED " << m << endl;
	int encoded[m];
	for(int i = 0; i < m; i++) {
		encoded[i] = sent[i];
	}
	shuffle(encoded, encoded + m, mt19937(chrono::steady_clock::now().time_since_epoch().count()));
	decode(n, m, encoded);
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