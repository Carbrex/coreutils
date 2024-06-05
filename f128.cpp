#include <quadmath.h>
#include <iostream>

int main() {
    __float128 f128_var = 11e4931Q;
    char buf[1000];
    quadmath_snprintf(buf, sizeof buf, "%.1000Qe", f128_var);
    std::cout << buf << std::endl;
    return 0;
}