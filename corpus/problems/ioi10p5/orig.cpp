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

char faceup(int C) {
    printf("%d\n", C);
    fflush(stdout);
    char c;
    if (scanf(" %c", &c) != 1) exit(1);
    return c;
}
void play() {
    vector<int> arr[25];
    for (int i = 1; i <= 50; i++) {
        arr[faceup(i) - 'A'].push_back(i);
    }
    for (auto& a : arr) {
        faceup(a[0]);
        faceup(a[1]);
    }
}
int main() {
    play();
    printf("0\n");
}
