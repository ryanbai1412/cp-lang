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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void WriteAdvice(unsigned char a);

void write(int bits, int x) {
    for (int i = 0; i < bits; i++) {
        WriteAdvice((x >> i) & 1);
    }
}

void ComputeAdvice(int* arr, int n, int k, int) {
    vector<int> inds[n];
    for (int i = 0; i < n; i++) {
        inds[arr[i]].push_back(i);
    }
    for (auto& a : inds) {
        a.push_back(n);
        reverse(begin(a), end(a));
    }
    int in[n];
    memset(in, -1, sizeof(in));
    iota(in, in + k, 0);
    bool vis[n + k] {};
    set<pair<int, int>> pq;
    for (int i = 0; i < k; i++) {
        pq.emplace(inds[i].back(), i);
    }
    int ans = 0;
    for (int i = 0; i < n; i++) {
        int cx = inds[arr[i]].back();
        inds[arr[i]].pop_back();
        if (in[arr[i]] != -1) {
            vis[in[arr[i]]] = true;
            in[arr[i]] = i + k;
            pq.erase({cx, arr[i]});
            pq.emplace(inds[arr[i]].back(), arr[i]);
            continue;
        }
        int x = (--pq.end())->second;
        pq.erase(--pq.end());
        in[x] = -1;
        in[arr[i]] = i + k;
        pq.emplace(inds[arr[i]].back(), arr[i]);
        ans++;
    }
    for (auto& a : vis) {
        WriteAdvice(a);
    }
}

int GetRequest();
void PutBack(int T);

int read(unsigned char*& ptr, int bits) {
    int x = 0;
    for (int i = 0; i < bits; i++) {
        x |= *(ptr++) << i;
    }
    return x;
}

void Assist(unsigned char* buf, int n, int k, int) {
    bool vis[n] {};
    vector<int> unused;
    for (int i = 0; i < k; i++) {
        vis[i] = true;
        if (!read(buf, 1)) {
            unused.push_back(i);
        }
    }
    for (int i = 0; i < n; i++) {
        int cur = GetRequest();
        if (!vis[cur]) {
            int u = unused.back();
            unused.pop_back();
            PutBack(u);
            vis[u] = false;
            vis[cur] = true;
        }
        if (!read(buf, 1)) {
            unused.push_back(cur);
        }
    }
}// The original is the two functions the IOI grader calls. This `main` stands in for the
// grader: n k, then the n requests; it prints ComputeAdvice's bits on one line, then the
// colour Assist puts back at each miss.
static vector<unsigned char> advice;
static vector<int> requests, put_back;
static size_t next_request = 0;
void WriteAdvice(unsigned char a) { advice.push_back(a); }
int GetRequest() { return requests[next_request++]; }
void PutBack(int T) { put_back.push_back(T); }
int main() {
	int n, k;
	if (scanf("%d %d", &n, &k) != 2) return 1;
	requests.resize(n);
	for (int& c : requests) scanf("%d", &c);
	vector<int> arr = requests;
	ComputeAdvice(arr.data(), n, k, n + k);
	for (size_t i = 0; i < advice.size(); i++) printf("%s%d", i ? " " : "", advice[i]);
	printf("\n");
	vector<unsigned char> buf = advice;
	Assist(buf.data(), n, k, (int)buf.size());
	for (size_t i = 0; i < put_back.size(); i++) printf("%s%d", i ? " " : "", put_back[i]);
	printf("\n");
}
