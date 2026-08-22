HOST_BIND ?= 127.0.0.1
HOST_PORT ?= 4176
WEB_PORT ?= 4177

.PHONY: start desk host dev test check

# One process. One URL. This is the product.
start: desk
	@echo "Open http://$(HOST_BIND):$(HOST_PORT)"
	cargo run --quiet -- serve --bind $(HOST_BIND) --port $(HOST_PORT) --web-dir web/dist --testdata testdata

desk: web/node_modules
	cd web && npm run build

host:
	cargo run -- serve --bind $(HOST_BIND) --port $(HOST_PORT)

# Desk HMR on 4177, host API on 4176.
dev: web/node_modules
	@echo "QuantForge host $(HOST_BIND):$(HOST_PORT)  UI http://$(HOST_BIND):$(WEB_PORT)"
	@cargo build
	@cargo run --quiet -- serve --bind $(HOST_BIND) --port $(HOST_PORT) --testdata testdata & \
		host_pid=$$!; \
		trap 'kill $$host_pid' EXIT INT TERM; \
		cd web && npm run dev -- --host $(HOST_BIND) --port $(WEB_PORT)

web/node_modules: web/package.json web/package-lock.json
	cd web && npm ci

test check:
	cargo test
	cd web && npm run check
