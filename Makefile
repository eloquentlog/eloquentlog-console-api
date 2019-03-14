# setup -- {{{
setup\:tools:  ## Setup development tools
	cargo install diesel_cli --no-default-features --features "postgres"
.PHONY: setup\:tools
# }}}

# vet -- {{{
vet\:check:  ## Check rust syntax [alias: check]
	@cargo check --all -v
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
	@cargo test --test top_test --test assets_test --test errors_test
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

# other utilities -- {{{
clean:  ## Tidy up
	@rm -fr vendor
	@cargo clean
.PHONY: clean

watch:  ## Start watch process for development
	@cargo watch -x 'run' -d 0.3
.PHONY: watch

help:  ## Display this message
	@grep -E '^[0-9a-z\:\\]+: ' $(MAKEFILE_LIST) | \
	  grep -E '  ## ' | \
	  sed -e 's/\(\s|\(\s[0-9a-z\:\\]*\)*\)  /  /' | \
	  tr -d \\\\ | \
	  awk 'BEGIN {FS = ":  ## "}; \
	      {printf "\033[38;05;222m%-17s\033[0m %s\n", $$1, $$2}' | \
	  sort
.PHONY: help
# }}}

.DEFAULT_GOAL = test\:all
default: test\:all
