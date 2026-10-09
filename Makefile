# Solar Realms Elite reconstruction.  Builds ./sre with the system C++
# compiler (clang on macOS).
CXX ?= c++
CXXFLAGS ?= -O2 -g
CXXFLAGS += -std=c++11 -fwrapv -fno-strict-aliasing -Wall -Wno-unused-variable -Wno-unused-but-set-variable
SRCS := $(wildcard src/*.cpp)
OBJS := $(SRCS:src/%.cpp=build/%.o)

sre: $(OBJS)
	$(CXX) $(CXXFLAGS) -o $@ $(OBJS)

build/%.o: src/%.cpp src/*.h | build
	$(CXX) $(CXXFLAGS) -c -o $@ $<

build:
	mkdir -p build

clean:
	rm -rf build sre

.PHONY: clean
