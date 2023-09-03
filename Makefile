DATABASE_URL := $(if $(ENV),"$$$(shell echo "$(ENV)" | \
	tr '[:lower:]' '[:upper:]')_DATABASE_URL","$$DATABASE_URL")
MIGRATION_DIRECTORY := migration
MIGRATION_NAME ?=

COVERAGE := 'covered":"([0-9]*\.[0-9]*|[0-9]*)"' | sed -E 's/[a-z\:"]*//g'

NAMESPACE := eloquentlog
PACKAGE := $(NAMESPACE)-console-api
ENV := development

# setup
setup\:vendor: ## Install cargo vendor and run it
	@mkdir -p .cargo
	@cargo vendor > .cargo/config
.PHONY: setup\:vendor

# https://github.com/rust-lang/cargo/issues/2267
setup\:tool: ## Install development tools
	@mkdir -p .git/hooks
	@which diesel >/dev/null 2>&1 || cargo install \
		diesel_cli \
		--version 1.4.1 \
		--no-default-features \
		--features "postgres" \
		--locked \
		--force
.PHONY: setup\:tool

setup\:all: setup\:tool setup\:vendor ## Setup vendor and tool both
.PHONY: setup\:all

setup: setup\:vendor ## Alias of setup:vendor
.PHONY: setup

# vet
vet\:check: ## Check Rust syntax [synonym: check]
	@cargo check --all --verbose
.PHONY: vet\:check

check: vet\:check
.PHONY: check

vet\:format: ## Check format without changes [synonym: vet:fmt, format, fmt]
	@cargo fmt --all -- --check
.PHONY: vet\:format

vet\:fmt: vet\:format
.PHONY: vet\:fmt

format: vet\:format
.PHONY: format

fmt: vet\:format
.PHONY: fmt

vet\:lint: ## Check style using clippy [synonym: lint]
	@cargo clippy --all-targets
.PHONY: vet\:lint

lint: vet\:lint
.PHONY: lint

vet\:all: vet\:check vet\:format vet\:lint ## Check code using all vet targets
.PHONY: vet\:all

vet: vet\:check ## Alias of vet:check
.PHONY: vet

# test
test\:lib: ## Run tests for lib [synonym: test]
	@cargo test --lib
.PHONY: test\:lib

test\:e2e: ## Run e2e tests
	@cargo test --test e2e
.PHONY: test\:e2e

test\:doc: ## Run doc tests
	@cargo test --doc
.PHONY: test\:doc

test\:all: test\:doc ## Run tests for doc, lib and e2e
	@cargo test --lib --test e2e
.PHONY: test\:all

test: test\:lib
.PHONY: test

# coverage
_get_covered:
	result=($(DST_DIR)/index.js*); \
	if [ -f $${result}[0] ]; then \
		rm "$(DST_DIR)/index.js*"; \
	fi; \
	file=($(DST_DIR)/debug/deps/$(MODULE)-*); \
	kcov --verify --include-path=$(SRC_DIR) $(DST_DIR) $${file[0]}; \
	grep 'index.html' $(DST_DIR)/index.js* | \
		grep --only-matching --extended-regexp $(COVERAGE)

coverage\:lib: ## Generate a coverage report of tests for library [synonym: cov:lib]
	@set -uo pipefail; \
	dir="$$(pwd)"; \
	target_dir="$${dir}/target/coverage/lib"; \
	cargo test --lib --no-run --target-dir=$${target_dir}; \
	make -s SRC_DIR=$${dir}/src DST_DIR=$${target_dir} \
		MODULE=$(NAMESPACE)_console_api _get_covered
.PHONY: coverage\:lib

cov\:lib: coverage\:lib
.PHONY: cov\:lib

# NOTE:
# e2e requires also an actual application binary of server under the
# target/debug/deps directory.
coverage\:e2e: ## Generate a coverage report of e2e tests [synonym: cov:e2e]
	@set -uo pipefail; \
	dir="$$(pwd)"; \
	target_dir="$${dir}/target/coverage/e2e"; \
	export CARGO_TARGET_DIR=$${target_dir}; \
	cargo test --test e2e --no-run --target-dir=$${target_dir}; \
	make -s SRC_DIR=$${dir}/src DST_DIR=$${target_dir} \
		MODULE=e2e _get_covered
.PHONY: coverage\:e2e

cov\:e2e: coverage\:e2e
.PHONY: cov\:e2e

coverage\:all: coverage\:lib coverage\:e2e ## Generated merged coverage report of all tests [synonym: cov:all]
	@set -uo pipefail; \
	dir="$$(pwd)"; \
	output_dir="$${dir}/target/coverage"; \
	kcov --merge $${output_dir} $$output_dir/lib $$output_dir/e2e; \
	grep '\[merged\]' $$output_dir/index.js* | \
		grep --only-matching --extended-regexp $(COVERAGE)
.PHONY: coverage\:all

cov\:all: coverage\:all
.PHONY: cov\:all

coverage: coverage\:lib ## Alias of coverage:lib [synonym: cov]
.PHONY: cov

cov: coverage
.PHONY: cov

# build
build\:debug: ## build targets in debug mode [synonym: debug]
	cargo build
.PHONY: build\:debug

debug: build\:debug
.PHONY: debug

build: build\:debug ## Alias of build:debug
.PHONY: build

build\:release: ## Build targets in release mode [synonym: release]
	cargo build --release
.PHONY: build\:release

release: build\:release
.PHONY: release

build\:debug\:server: ## build only server binary in debug mode
	cargo build --bin $(PACKAGE)-server
.PHONY: build\:debug\:server

build\:server: build\:debug\:server ## Alias of build:debug:server
.PHONY: build\:server

build\:release\:server: ## build only server binary in release mode
	cargo build --bin $(PACKAGE)-server --release
.PHONY: build\:release\:server

build\:debug\:worker: ## build only worker binary in debug mode
	cargo build --bin $(PACKAGE)-worker
.PHONY: build\:debug\:worker

build\:worker: build\:debug\:worker ## Alias of build:debug:worker
.PHONY: build\:worker

build\:release\:worker: ## build only worker binary in release mode
	cargo build --bin $(PACKAGE)-worker --release
.PHONY: build\:release\:worker

build\:debug\:router: ## build only router binary in debug mode
	cargo build --bin $(PACKAGE)-router
.PHONY: build\:debug\:router

build\:router: build\:debug\:router ## Alias of build:debug:router
.PHONY: build\:router

build\:release\:router: ## build only router binary in release mode
	cargo build --bin $(PACKAGE)-router --release
.PHONY: build\:release\:router

# utility
watch\:server: ## Start watch process for development server [synonym: server]
	@cargo watch --exec 'run --bin $(PACKAGE)-server' --delay 0.3 \
		--ignore '(\.tool|tmp|migration|src\/worker)/\*'
.PHONY: watch\:server

server: watch\:server
.PHONY: server

watch\:worker: ## Start watch process for development worker [synonym: worker]
	@cargo watch --exec 'run --bin $(PACKAGE)-worker' --delay 0.3 \
		--ignore '(\.tool|tmp|migration|src\/server)/\*'
.PHONY: watch\:worker

worker: watch\:worker
.PHONY: worker

watch\:check: ## Start watch process for check
	@cargo watch --postpone --exec 'check --all --verbose'
.PHONY: watch\:check

watch\:fmt: ## Start watch process for fmt
	@cargo watch --postpone --exec 'fmt --all -- --check'
.PHONY: watch\:fmt

watch\:lint: ## Start watch process for lint
	@cargo watch --postpone --exec 'clippy --all-targets'
.PHONY: watch\:lint

watch\:test\:lib: ## Start watch process for test:lib
	@cargo watch --postpone --exec 'test --lib'
.PHONY: watch\:test\:lib

watch\:test\:e2e: ## Start watch process for test:e2e
	@cargo watch --postpone --exec 'test --test e2e'
.PHONY: watch\:test\:e2e

watch\:test: watch\:test\:lib ## Alias of watch:test:lib
.PHONY: watch\:test

watch: watch\:server ## Alias of watch:server
.PHONY: watch

schema\:migration\:commit: ## Run all migrations
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration run --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:commit

schema\:migration\:create: ## Generate new migration files (require: MIGRATION_NAME env var)
	@if [ -z "$(MIGRATION_NAME)" ]; then \
		echo "You need to set \$$MIGRATION_NAME, e.g. \`MIGRATION_NAME=xxx make ...\`"; \
		exit 1; \
	fi
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration generate \
		--migration-dir $(MIGRATION_DIRECTORY) \
		--version $$(date +%Y%m%d%H%M%S) \
		$(MIGRATION_NAME)
.PHONY: schema\:migration\:create

schema\:migration\:revert: ## Rollback a latest migration
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(DATABASE_URL)"; \
	diesel migration revert --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:revert

schema\:migration\:status: ## List migrations
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(#|$$)"); \
	fi; \
	export DATABASE_URL="$(DATABASE_URL)"; \
	diesel migration list --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:status

document\:er: ## Generate & display an ER diagram [synonym: doc:er]
	@dot -T png doc/er.dot > doc/er.png; feh doc/er.png
.PHONY: document\:er

doc\:er: document\:er
.PHONY: doc\:er

document\:lib: ## Generate doc for lib [synonym: doc:lib, document, doc]
	@cargo doc --lib --no-deps
.PHONY: document\:lib

doc\:lib: document\:lib
.PHONY: doc\:lib

document: document\:lib
.PHONY: document

doc: document
.PHONY: doc

clean: ## Tidy up
	@rm --force --recursive vendor
	@cargo clean
.PHONY: clean

route: ## Print all routes using router
	@cargo run --bin $(PACKAGE)-router
.PHONY: route

runner-%: ## Run a CI job on local (on Docker)
	@set -uo pipefail; \
	job=$(subst runner-,,$@); \
	opt=""; \
	while read line; do \
		opt+=" --env $$(echo $$line | sed -E 's/^export //')"; \
	done < .env.ci; \
	gitlab-runner exec docker \
		--executor docker \
		--cache-dir /cache \
		--docker-volumes $$(pwd)/.cache/gitlab-runner:/cache \
		--docker-volumes /var/run/docker.sock:/var/run/docker.sock \
		$${opt} $${job}
.PHONY: runner

help: ## Display this message
	@set -uo pipefail; \
	grep --extended-regexp '^[-_0-9a-z\%\:\\ ]+: ' \
		$(firstword $(MAKEFILE_LIST)) | \
		grep --extended-regexp ' ## ' | \
		sed --expression='s/\( [-_0-9a-z\%\:\\ ]*\) #/ #/' | \
		tr --delete \\\\ | \
		awk 'BEGIN {FS = ": ## "}; \
			{printf "\033[38;05;222m%-24s\033[0m %s\n", $$1, $$2}' | \
		sort
.PHONY: help

.DEFAULT_GOAL = test\:all
default: test\:all
