# Solar Realms Elite reconstruction, Rust port.  Builds ./sre with Cargo.
# `make server` builds server/sre-server, the web front end, with Go.
CARGO ?= cargo

sre: Cargo.toml $(wildcard src/*.rs)
	$(CARGO) build --release
	cp target/release/sre $@

server: server/sre-server

server/sre-server: $(wildcard server/*.go server/web/* server/web/vendor/*) server/go.mod server/go.sum
	cd server && go build -o sre-server .

clean:
	$(CARGO) clean
	rm -f sre server/sre-server

.PHONY: clean server
