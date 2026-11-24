// The judge's side for the accepted C++: reads n w r from interactor.py, calls
// restore_permutation, and speaks `A s`, `C`, `Q s` and `! p` on stdio.
#include <iostream>
#include <string>

int* restore_permutation(int n, int w, int r);

static int reply() {
    int x;
    std::cin >> x;
    return x;
}

void add_element(std::string x) {
    std::cout << "A " << x << std::endl;
    reply();
}

void compile_set() {
    std::cout << "C" << std::endl;
    reply();
}

bool check_element(std::string x) {
    std::cout << "Q " << x << std::endl;
    return reply() == 1;
}

int main() {
    int n, w, r;
    std::cin >> n >> w >> r;
    int* p = restore_permutation(n, w, r);
    std::cout << "!";
    for (int i = 0; i < n; i++)
        std::cout << " " << p[i];
    std::cout << std::endl;
}
