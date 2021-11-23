
SHELL		= bash


#
# Project
#
preview-crate:			test-debug
	cargo publish --dry-run
publish-crate:			test-debug
	CARGO_HOME=$(HOME)/.cargo cargo publish


#
# Testing
#
tests/package-lock.json:	tests/package.json
	touch $@
tests/node_modules:		tests/package-lock.json
	cd tests; npm install
	touch $@
test:				test-unit test-integration
test-debug:			test-unit-debug test-integration-debug
test-setup:			tests/node_modules

test-unit:
	cargo test --quiet --tests
test-unit-debug:
	RUST_BACKTRACE=1 cargo test --tests -- --nocapture --show-output

DNA_NAME			= happy_path
TEST_DNA			= tests/dnas/$(DNA_NAME).dna
TEST_DNA_WASM			= tests/zomes/$(DNA_NAME).wasm

tests/dnas/%.dna:		tests/dnas/%/dna.yaml tests/zomes/%.wasm
	hc dna pack -o $@ tests/dnas/$*/

tests/zomes/%.wasm:		tests/zomes/%/src/*.rs tests/zomes/%/Cargo.toml Cargo.toml src/*.rs
	cd tests/zomes/; RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*
	mv tests/zomes/target/wasm32-unknown-unknown/release/$*.wasm $@

test-integration:		test-setup $(TEST_DNA)
	cd tests; npx mocha integration/test_basic.js
test-integration-debug:		test-setup $(TEST_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=silly npx mocha integration/test_basic.js


#
# Documentation
#
test-docs:
	cargo test --doc
build-docs:			test-docs
	cargo doc


#
# Repository
#
clean-remove-chaff:
	@find . -name '*~' -exec rm {} \;
clean-files:		clean-remove-chaff
	git clean -nd
clean-files-force:	clean-remove-chaff
	git clean -fd
clean-files-all:	clean-remove-chaff
	git clean -ndx
clean-files-all-force:	clean-remove-chaff
	git clean -fdx
