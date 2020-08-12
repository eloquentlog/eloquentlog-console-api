# schema (diesel)
VAR_DATABASE_URL := $(if $(ENV),"$$$(shell echo "$(ENV)" | \
	tr '[:lower:]' '[:upper:]')_DATABASE_URL","$$DATABASE_URL")
MIGRATION_DIRECTORY := migration
MIGRATION_NAME ?=

APP := eloquentlog-console-api
ENV := development

# deployment
GCP_PROJECT_ID ?=
GCP_CLOUD_BUILD_CREDENTIAL_JSON ?=
GCP_CLOUD_BUILD_SUBSTR_ENV_VARS ?=
GCP_CLOUD_STORAGE_LOG_DIRECTORY ?=

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
verify\:check:  ## Check Rust syntax [alias: check]
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
coverage:  ## Generate coverage report of unit tests only for lib using Kcov [alias: cov]
	@cargo test --lib --no-run
	@./.tool/setup-kcov
	./.tool/get-covered eloquentlog_console_api
.PHONY: coverage

cov: | coverage
.PHONY: cov
# }}}

# build -- {{{
build\:debug:  ## build targets in debug mode [alias: build]
	cargo build
.PHONY: build\:debug

build: | build\:debug
.PHONY: build

build\:release:  ## Build targets in release mode
	cargo build --release
.PHONY: build\:release

## applications

build\:debug\:server:  ## build only server binary in debug mode [alias: build:server]
	cargo build --bin server
.PHONY: build\:debug\:server

build\:server: | build\:debug\:server
.PHONY: build\:server

build\:release\:server:  ## build only server binary in release mode
	cargo build --bin server --release
.PHONY: build\:release\:server

build\:debug\:worker:  ## build only worker binary in debug mode [alias: build:worker]
	cargo build --bin worker
.PHONY: build\:debug\:worker

build\:worker: | build\:debug\:worker
.PHONY: build\:worker

build\:release\:worker:  ## build only worker binary in release mode
	cargo build --bin worker --release
.PHONY: build\:release\:worker

## utilities

build\:debug\:router:  ## build only router binary in debug mode [alias: build:router]
	cargo build --bin router
.PHONY: build\:debug\:router

build\:router: | build\:debug\:router
.PHONY: build\:router

build\:release\:router:  ## build only router binary in release mode
	cargo build --bin router --release
.PHONY: build\:release\:router
# }}}

# watch -- {{{
watch\:server:  ## Start watch process for development server [alias: server]
	@cargo watch --exec 'run --bin server' --delay 0.3 \
		--ignore '(\.tool|tmp|migration|src\/worker)/\*'
.PHONY: watch\:server

server: | watch\:server
.PHONY: server

watch\:worker:  ## Start watch process for development worker [alias: worker]
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
schema\:migration\:commit:  ## Run all migrations
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(\#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration run --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:commit

schema\:migration\:create:  ## Generate new migration files (require: MIGRATION_NAME env var)
	@if [ -z "$(MIGRATION_NAME)" ]; then \
		echo "You need to set \$$MIGRATION_NAME, e.g. \`MIGRATION_NAME=xxx make ...\`"; \
		exit 1; \
	fi
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(\#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel setup --migration-dir $(MIGRATION_DIRECTORY) && \
	diesel migration generate \
		--migration-dir $(MIGRATION_DIRECTORY) \
		--version $$(date +%Y%m%d%H%M%S) \
		$(MIGRATION_NAME)
.PHONY: schema\:migration\:create

schema\:migration\:revert:  ## Rollback a latest migration
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(\#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel migration revert --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:revert

schema\:migration\:status:  ## List migrations
	@if [ -f "$$(pwd)/.env" ]; then \
		source $$(pwd)/.env && \
		export $$(cut -d "=" -f 1 $$(pwd)/.env | grep -vE "^(\#|$$)"); \
	fi; \
	export DATABASE_URL="$(VAR_DATABASE_URL)"; \
	diesel migration list --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:status
# }}}

# deploy -- {{{
_deploy-%:
	@set -euo pipefail; \
	BUILD_TARGET="$(subst _deploy-,,$@)"; \
	if [ "$${BUILD_TARGET}" != "server" ] && \
		[ "$${BUILD_TARGET}" != "worker" ]; then \
		exit; \
	fi; \
	export CLOUDSDK_PYTHON=python3.7; \
	export CLOUDSDK_CORE_PROJECT="$(GCP_PROJECT_ID)"; \
	gcloud auth activate-service-account \
		--key-file=$(GCP_CLOUD_BUILD_CREDENTIAL_JSON); \
	SUBSTITUTIONS="$$( \
		cat $(GCP_CLOUD_BUILD_SUBSTR_ENV_VARS) | \
		grep '^_' | \
		sed -e :a -e 'N;s/\n/,/;ta' | \
		sed -e 's/\"//g' \
	)"; \
	SUBSTITUTIONS=$$( \
		printf " \
		_SERVICE_NAME=$(APP)-$${BUILD_TARGET}, \
		_BUILD_TARGET_NAME=$${BUILD_TARGET}, \
		_BUILD_LOGS_BUCKET=$(GCP_CLOUD_STORAGE_LOG_DIRECTORY), \
		%s" \
		"$${SUBSTITUTIONS}" | sed 's/ //g' \
	); \
	gcloud builds submit \
		--config=.build.$${BUILD_TARGET}.yml . \
		--substitutions="$${SUBSTITUTIONS}"

deploy\:server: _deploy-server  ## Deploy `server` on a cluster on Cloud Run (require: GCP_XXX env vars)
.PHONY: deploy\:server

deploy\:worker: | _deploy-worker  ## Deploy `worker` container on Compute Engine (require: GCP_XXX env vars)
.PHONY: deploy\:worker
# }}}

# other utilities -- {{{
clean:  ## Tidy up
	@rm --force --recursive vendor
	@cargo clean
.PHONY: clean

doc\:er:  ## Generate & display an ER diagram
	@dot -T png doc/er.dot > doc/er.png; feh doc/er.png
.PHONY: doc\:er

doc\:lib:  ## Generate doc for lib [alias: doc]
	@cargo doc --lib --no-deps
.PHONY: doc\:lib

doc: | doc\:lib
.PHONY: doc

route:  ## Print all routes using router
	@cargo run --bin router
.PHONY: route

help:  ## Display this message
	@grep --extended-regexp '^[0-9a-z\:\\\%]+: ' $(MAKEFILE_LIST) | \
		grep --extended-regexp '  ## ' | \
		sed --expression='s/\(\s|\(\s[-_0-9a-z\:\\]*\)*\)  /  /' | \
		tr --delete \\\\ | \
		awk 'BEGIN {FS = ":  ## "}; \
			{printf "\033[38;05;222m%-24s\033[0m %s\n", $$1, $$2}' | \
		sort
.PHONY: help
# }}}

.DEFAULT_GOAL = test\:all
default: test\:all
