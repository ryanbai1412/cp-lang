#include <cstdio>
#include <string>
#include <vector>

int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<std::string> words(n);
    static char buf[64];
    for (auto& w : words) {
        if (scanf("%63s", buf) != 1) return 1;
        w = buf;
    }
    std::vector<std::vector<unsigned char>> lists(n);
    for (int i = 0; i < n; i++) {
        lists[i].reserve(words[i].size());
        for (char c : words[i]) lists[i].push_back((unsigned char)(c - 'a'));
    }
    long long t = 0;
    for (auto& w : lists)
        for (unsigned char c : w) t = (t * 31 + c) % 1000000007;
    printf("%lld\n", t);
}
