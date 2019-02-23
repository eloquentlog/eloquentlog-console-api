# vet {{{
vet\:check:  ## Check rust syntax
	cargo check --all -v
.PHONY: vet\:check

vet\:format:  ## Check format without changes (alias: vet:fmt, fmt)
	cargo fmt --all -- --check
.PHONY: format

vet\:fmt: | vet\:format
.PHONY: vet\:fmt

fmt: | vet\:format
.PHONY: fmt

vet\:lint:  ## Check code style using clippy (alias: lint)
	cargo clippy --all-targets
.PHONY: vet\:lint

lint: | vet\:lint
.PHONY: lint

vet\:all: | vet\:check vet\:format vet\:lint  ## Check by all vet:xxx targets
.PHONY: vet\:all

vet: | vet\:check  ## Same as vet:all
.PHONY: vet
# }}}

# test {{{
test\:unit:  ## Run only unit tests
	cargo test --lib
.PHONY: test\:unit

test\:integration:  ## Run integrations test only
	cargo test --test top_test --test assets_test --test errors_test
.PHONY: test\:integration

test\:all:  ## Run all test targets
	cargo test --tests
.PHONY: test\:all

test: | test\:all   ## Same as test:all
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

# build {{{
build\:debug:  ## Create debug build
	cargo build
.PHONY: build\:debug

build\:release:  ## Create release build
	cargo build --release
.PHONY: build\:release

build: | build\:debug  ## Same as build:debug
.PHONY: build
# }}}

# other utilities {{{
clean:  ## Clean up
	cargo clean
.PHONY: clean

watch:  ## Start watch process for development
	cargo watch -x 'run' -d 0.3
.PHONY: watch

help:  ## Display this message
	@grep -E '^[0-9a-z\:\\]+: ' $(MAKEFILE_LIST) | grep -E '  ## ' | \
	  sed -e 's/\(\s|\(\s[0-9a-z\:\\]*\)*\)  /  /' | tr -d \\\\ | \
	  awk 'BEGIN {FS = ":  ## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' | \
	  sort
.PHONY: help

.DEFAULT_GOAL = test\:all
default: vet\:check vet\:format vet\:lint test\:all
# }}}
