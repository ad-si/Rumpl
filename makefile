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


.PHONY: instruments
instruments: build
	@mkdir -p examples/instruments
	@for prog in $$(seq 0 127); do \
		./target/release/rumpl demo $$prog examples/instruments; \
	done


.PHONY: examples
examples: build
	@for mid in examples/instruments/*.mid \
			examples/songs/*.mid examples/songs/*.kar examples/songs/*.rmi; do \
		[ -e "$$mid" ] || continue; \
		out="$${mid%.*}.opus"; wav="$${mid%.*}.wav"; \
		if ! ./target/release/rumpl render "$$mid" "$$wav" >/dev/null 2>&1; then \
			echo "skip (render failed): $$mid"; continue; \
		fi; \
		ffmpeg -y -loglevel error -i "$$wav" -ac 1 -c:a libopus -b:a 64k "$$out" </dev/null; \
		rm -f "$$wav"; \
		echo "rendered $$mid → $$out"; \
	done


.PHONY: install
install:
	cargo install --path .


.PHONY: clean
clean:
	cargo clean
