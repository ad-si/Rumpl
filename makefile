.PHONY: help
help: makefile
	@tail -n +4 makefile | grep ".PHONY"


.PHONY: format
format:
	cargo clippy --fix --allow-dirty --allow-staged --allow-no-vcs
	cargo fmt


.PHONY: build
build:
	cargo build --release


.PHONY: test
test:
	cargo test -- --show-output


.PHONY: install
install:
	cargo install --path .


.PHONY: clean
clean:
	cargo clean
