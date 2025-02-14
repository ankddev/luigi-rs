#ifdef _WIN32
    #define UI_WINDOWS
#elif __linux__
    #define UI_LINUX
#else
    #error "Unsupported platform. Only Windows and Linux are supported."
#endif

#define UI_IMPLEMENTATION
#include "../luigi.h"
