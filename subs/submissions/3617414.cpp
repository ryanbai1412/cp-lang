#include "bits/extc++.h"

using namespace std;

template<class T>
void dbgh(T t) {
	cerr << t << endl;
}

template<class T, class... U>
void dbgh(T t, U... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__); cerr
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

#define long int64_t
#define sz(x) static_cast<int>((x).size())

const int maxn = 50;
const long nans = 4e18;

int n;
string arr[maxn];
map<string, long> memo;

void solve() {
	int m;
	cin >> n >> m;
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	int ind = 0;
	string conv[n * m];
	auto get = [&] (const string &s) {
		auto it = memo.find(s);
		if(it == memo.end()) {
			conv[ind] = s;
			return memo[s] = ind++;
		}
		return it->second;
	};
	get("");
	long dist[n * m];
	memset(dist, 0x7f, sizeof(dist));
	priority_queue<pair<int, int>, vector<pair<int, int>>, greater<>> pq;
	auto put = [&] (const string &s, int d) {
		int cind = get(s);
		if(d < dist[cind]) {
			dist[cind] = d;
			pq.emplace(d, cind);
		}
	};
	for(int i = 0; i < n; i++) {
		for(int j = i + 1; j < n; j++) {
			string a = arr[i], b = arr[j];
			int nsz = min(sz(a), sz(b));
			if(a.substr(0, nsz) == b.substr(0, nsz)) {
				if(sz(a) > sz(b)) {
					swap(a, b);
				}
				string ns = b.substr(sz(a));
				put(ns, sz(a));
			}
		}
	}
	while(sz(pq)) {
		auto [d, u] = pq.top();
		pq.pop();
		if(d == dist[u]) {
			if(u == 0) {
				cout << d << endl;
				return;
			}
			string s = conv[u];
			dbg(u, d, s);
			for(int i = 0; i < n; i++) {
				string a = s, b = arr[i];
				int nsz = min(sz(a), sz(b));
				if(a.substr(0, nsz) == b.substr(0, nsz)) {
					if(sz(a) > sz(b)) {
						swap(a, b);
					}
					string ns = b.substr(sz(a));
					put(ns, d + sz(a));
				}
			}
		}
	}
	cout << -1 << endl;
}

int main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}