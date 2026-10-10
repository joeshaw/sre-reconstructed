# Solar Realms Elite reconstruction.  Builds ./sre with the system C++
# compiler (clang on macOS).  `make server` builds server/sre-server, the
# web front end, with Go.
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

server: server/sre-server

server/sre-server: $(wildcard server/*.go server/web/* server/web/vendor/*) server/go.mod server/go.sum
	cd server && go build -o sre-server .

clean:
	rm -rf build sre server/sre-server

.PHONY: clean server
