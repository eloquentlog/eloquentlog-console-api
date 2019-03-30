MIGRATION_DIRECTORY := migration

# setup -- {{{
setup\:tools:  ## Setup development tools
	cargo install diesel_cli --no-default-features --features "postgres"
.PHONY: setup\:tools
# }}}

# vet -- {{{
vet\:check:  ## Check rust syntax [alias: check]
	@cargo check --all --verbose
.PHONY: vet\:check

check: | vet\:check
.PHONY: check

vet\:format:  ## Check format without changes [alias: vet:fmt, format, fmt]
	@cargo fmt --all -- --check
.PHONY: vet\:format

vet\:fmt: | vet\:format
.PHONY: vet\:fmt

format: | vet\:format
.PHONY: format

fmt: | vet\:format
.PHONY: fmt

vet\:lint:  ## Check style using clippy [alias: lint]
	@cargo clippy --all-targets
.PHONY: vet\:lint

lint: | vet\:lint
.PHONY: lint

vet\:all: | vet\:check vet\:format vet\:lint  ## Check code using all vet:xxx targets [alias: vet]
.PHONY: vet\:all

vet: | vet\:all
.PHONY: vet
# }}}

# test -- {{{
test\:unit:  ## Run unit tests
	@cargo test --lib
.PHONY: test\:unit

test\:integration:  ## Run integrations test only
	@cargo test --test integration
.PHONY: test\:integration

test\:all:  ## Run unit tests and integration tests [alias: test]
	@cargo test --tests
.PHONY: test\:all

test: | test\:all
.PHONY: test
# }}}

# coverage -- {{{
coverage:  ## Generate coverage report of unit tests only for lib using kcov [alias: cov]
	@cargo test --lib --no-run
	@./.tools/setup-kcov
	./.tools/get-covered eloquentlog_backend_api
.PHONY: coverage

cov: | coverage
.PHONY: cov
# }}}

# build -- {{{
build\:debug:  ## build debug [alias: debug]
	cargo build
.PHONY: build\:debug

build: | build\:debug
.PHONY: build

build\:release:  ## Build release
	cargo build --release
.PHONY: build\:release
# }}}

# watch -- {{{
watch:  ## Start watch process for development [alias: serve]
	@cargo watch --exec 'run' --delay 0.3 \
	  --ignore .tools/\* \
	  --ignore migration/\*
.PHONY: watch

serve: | watch
.PHONY: serve

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
	@diesel migration list --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:status

schema\:migration\:commit:  ## Run all migrations
	@diesel migration run --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:commit

schema\:migration\:revert:  ## Rollback a latest migration
	@diesel migration revert --migration-dir $(MIGRATION_DIRECTORY)
.PHONY: schema\:migration\:revert
# }}}

# other utilities -- {{{
clean:  ## Tidy up
	@rm --force --recursive vendor
	@cargo clean
.PHONY: clean

help:  ## Display this message
	@grep --extended-regexp '^[0-9a-z\:\\]+: ' $(MAKEFILE_LIST) | \
	  grep --extended-regexp '  ## ' | \
	  sed --expression='s/\(\s|\(\s[0-9a-z\:\\]*\)*\)  /  /' | \
	  tr --delete \\\\ | \
	  awk 'BEGIN {FS = ":  ## "}; \
	      {printf "\033[38;05;222m%-23s\033[0m %s\n", $$1, $$2}' | \
	  sort
.PHONY: help
# }}}

.DEFAULT_GOAL = test\:all
default: test\:all
