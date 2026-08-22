HOST_BIND ?= 127.0.0.1
HOST_PORT ?= 4176
WEB_PORT ?= 4177

.PHONY: start host web test check

start: web/node_modules
	@echo "QuantForge host $(HOST_BIND):$(HOST_PORT)  UI http://$(HOST_BIND):$(WEB_PORT)"
	@cargo build
	@QUANTFORGE_BIND=$(HOST_BIND) QUANTFORGE_PORT=$(HOST_PORT) \
		cargo run --quiet -- serve --bind $(HOST_BIND) --port $(HOST_PORT) & \
		host_pid=$$!; \
		trap 'kill $$host_pid' EXIT INT TERM; \
		cd web && npm run dev -- --host $(HOST_BIND) --port $(WEB_PORT)

host:
	cargo run -- serve --bind $(HOST_BIND) --port $(HOST_PORT)

web: web/node_modules
	cd web && npm run dev -- --host $(HOST_BIND) --port $(WEB_PORT)

web/node_modules: web/package.json
	cd web && npm install

test:
	cargo test
	cd web && npm run check

check:
	cargo test
	cd web && npm run check
