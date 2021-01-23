build: test
	@echo -e '\e[1;31mBuilding...\e[0m'
	@cd cli && \
		docker run --rm -v $$(pwd):/apy rust sh -c "cd apy && cargo build --release" && \
		cp ./target/release/awesome-persian-youtubers ..

test:
	@echo -e '\e[1;31mTesting...\e[0m'
	@cd cli && \
		cargo test

check:
	@cd cli && cargo +nightly fmt
	@cd cli && cargo clippy --all -- -D clippy::all -A clippy::field-reassign-with-default
	@cd cli && cargo +nightly udeps --all-targets
	@cd cli && cargo outdated -wR
	@cd cli && cargo update --dry-run
