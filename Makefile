VAR_DATABASE_URL := $(if $(ENV),"$$$(shell echo "$(ENV)" | \
	tr '[:lower:]' '[:upper:]')_DATABASE_URL","$$DATABASE_URL")

ENV := development

MIGRATION_DIRECTORY := migration

# setup -- {{{
setup\:vendor:  ## Install cargo vendor and run it
	@mkdir -p .cargo
	@which cargo-vendor >/dev/null 2>&1 || cargo install \
	  cargo-vendor --force
	@cargo vendor > .cargo/config
.PHONY: setup\:vendor

setup\:tool:  ## Install development tools
# for cargo-husky
	@mkdir -p .git/hooks
	@which diesel >/dev/null 2>&1 || cargo install \
	  diesel_cli --no-default-features --features "postgres" --force
.PHONY: setup\:tool

setup\:all: | setup\:tool setup\:vendor  ## Setup vendor and tool both [alias: setup]
.PHONY: setup\:all

setup: | setup\:all
.PHONY: setup
# }}}

# verify -- {{{
verify\:check:  ## Check rust syntax [alias: check]
	@cargo check --all --verbose
.PHONY: verify\:check

check: | verify\:check
.PHONY: check

verify\:format:  ## Check format without changes [alias: verify:fmt, format, fmt]
	@cargo fmt --all -- --check
.PHONY: verify\:format

verify\:fmt: | verify\:format
.PHONY: verify\:fmt

format: | verify\:format
.PHONY: format

fmt: | verify\:format
.PHONY: fmt

verify\:lint:  ## Check style using clippy [alias: lint]
	@cargo clippy --all-targets
.PHONY: verify\:lint

lint: | verify\:lint
.PHONY: lint

verify\:all: | verify\:check verify\:format verify\:lint  ## Check code using all verify:xxx targets [alias: verify]
.PHONY: verify\:all

verify: | verify\:all
.PHONY: verify
# }}}

# test -- {{{
test\:unit:  ## Run unit tests
	@cargo test --lib
.PHONY: test\:unit

test\:integration:  ## Run integration tests
	@cargo test --test integration
.PHONY: test\:integration

test\:doc:  ## Run doc tests
	@cargo test --doc
.PHONY: test\:doc

test\:all: | test\:doc  ## Run doc, unit and integration tests [alias: test]
	@cargo test --lib --test integration
.PHONY: test\:all

test: | test\:all
.PHONY: test
# }}}

# coverage -- {{{
coverage:  ## Generate coverage report of unit tests only for lib using kcov [alias: cov]
	@cargo test --lib --no-run
	@./.tool/setup-kcov
	./.tool/get-covered eloquentlog_backend_api
.PHONY: coverage

cov: | coverage
.PHONY: cov
# }}}

# build -- {{{
build\:debug:  ## build debug [alias: build]
	cargo build
.PHONY: build\:debug

build: | build\:debug
.PHONY: build

build\:release:  ## Build release
	cargo build --release
.PHONY: build\:release
# }}}

# watch -- {{{
watch\:server:  ## Start watch process for development server [alias: server]
	@cargo watch --exec 'run --bin server' --delay 0.3 \
	  --ignore '(\.tool|tmp|migration|src\/worker)/\*'
.PHONY: watch\:server

server: | watch\:server
.PHONY: server

watch\:worker:  ## Start watch process for development woker [alias: worker]
	@cargo watch --exec 'run --bin worker' --delay 0.3 \
	  --ignore '(\.tool|tmp|migration|src\/server)/\*'
.PHONY: watch\:worker

worker: | watch\:worker
.PHONY: worker

watch\:check:  ## Start watch process for check
	@cargo watch --postpone --exec 'check --all --verbose'
.PHONY: watch\:check

watch\:fmt:  ## Start watch process for fmt
	@cargo watch --postpone --exec 'fmt --all -- --check'
.PHONY: watch\:fmt

watch\:lint:  ## Start watch process for lint
	@cargo watch --postpone --exec 'clippy --all-targets'
.PHONY: watch\:lint

watch\:test\:unit:  ## Start watch process for test:unit
	@cargo watch --postpone --exec 'test --lib'
.PHONY: watch\:test\:unit

watch\:test\:integration:  ## Start watch process for test:integration
	@cargo watch --postpone --exec 'test --test integration'
.PHONY: watch\:test\:integration

watch\:test\:all:  ## Start watch process for test:all
	@cargo watch --postpone --exec 'test --tests'
.PHONY: watch\:test

watch\:test: | watch\:test\:all
.PHONY: watch\:test
# }}}

# schema -- {{{
schema\:migration\:status:  ## List migrations
	@if [ -f "$$(pwd)/.env" ]; then \
	  source $$(pwd)/.env && \
		export $$(cut -d= -f1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration list --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:status

schema\:migration\:commit:  ## Run all migrations
	@if [ -f "$$(pwd)/.env" ]; then \
	  source $$(pwd)/.env && \
		export $$(cut -d= -f1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration run --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:commit

schema\:migration\:revert:  ## Rollback a latest migration
	@if [ -f "$$(pwd)/.env" ]; then \
	  source $$(pwd)/.env && \
		export $$(cut -d= -f1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration revert --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:revert
# }}}

# other utilities -- {{{
clean:  ## Tidy up
	@rm --force --recursive vendor
	@cargo clean
.PHONY: clean

doc:  ## Generate doc for lib
	@cargo doc --lib --no-deps
.PHONY: doc

help:  ## Display this message
	@grep --extended-regexp '^[0-9a-z\:\\]+: ' $(MAKEFILE_LIST) | \
	  grep --extended-regexp '  ## ' | \
	  sed --expression='s/\(\s|\(\s[0-9a-z\:\\]*\)*\)  /  /' | \
	  tr --delete \\\\ | \
	  awk 'BEGIN {FS = ":  ## "}; \
	      {printf "\033[38;05;222m%-24s\033[0m %s\n", $$1, $$2}' | \
	  sort
.PHONY: help
# }}}

.DEFAULT_GOAL = test\:all
default: test\:all
