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

vector<int> tobinary(int x, int bits) {
	vector<int> v;
	for(int i = bits - 1; i >= 0; i--) {
		v.push_back((x >> i) & 1);
	}
	return v;
}

int frombinary(const vector<int> &x) {
	int ans = 0;
	for(auto &a: x) {
		ans = (ans << 1) | a;
	}
	return ans;
}

void encode(int n, int arr[]) {
	vector<int> v;
	for(int i = 0; i < n; i++) {
		auto cur = tobinary(arr[i], 8);
		v.insert(v.end(), begin(cur), end(cur));
	}
	while(sz(v) % 2) {
		v.push_back(0);
	}
	for(int i = 0; i < sz(v); i += 2) {
		vector<int> cur(v.begin() + i, v.begin() + i + 2);
		vector<int> id = tobinary(i / 2, 6);
		while(sz(id) < 8) {
			id.push_back(0);
		}
		assert(frombinary(cur) < 8);
		assert((frombinary(id) & 3) == 0);
		send(frombinary(id) | frombinary(cur));
	}
}

void decode(int n, int m, int arr[]) {
	vector<int> message[n * 4];
	for(int i = 0; i < m; i++) {
		vector<int> cur(tobinary(arr[i], 8));
		vector<int> id(cur.begin(), cur.begin() + 6);
		vector<int> val(cur.begin() + 6, cur.end());
		dbg(frombinary(id));
		assert(message[frombinary(id)].empty());
		message[frombinary(id)] = val;
	}
	vector<int> concat;
	for(auto &a: message) {
		concat.insert(concat.end(), begin(a), end(a));
	}
	for(int i = 0, ind = 0; i < n; i++, ind += 8) {
		output(frombinary(vector<int>(concat.begin() + ind, concat.begin() + ind + 8)));
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