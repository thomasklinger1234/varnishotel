RUST_CLIPPY_OPTS ?= --workspace --all-targets --all-features --locked -- -D warnings

DOCS_DEP_GROUP ?= docs
DOCS_SITE_DIR ?= site

rust-fmt:
	cargo fmt --all --check

rust-clippy:
	cargo clippy $(RUST_CLIPPY_OPTS)

rust-test:
	cargo test --all

docs-deps:
	uv sync --all-extras --group $(DOCS_DEP_GROUP)

docs-build:
	uv run mkdocs build --clean --site-dir $(DOCS_SITE_DIR)