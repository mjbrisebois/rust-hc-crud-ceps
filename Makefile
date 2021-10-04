
SHELL		= bash


#
# Project
#
tests/package-lock.json:	tests/package.json
	touch $@
tests/node_modules:		tests/package-lock.json
	cd tests; npm install
	touch $@
rebuild:			clean build
build:				dnarepo happdna

hc_crud_ceps:			hc_crud_ceps/src/*.rs
	cd $@; cargo build && touch $@
preview-crate:
	cargo publish --dry-run
publish-crate:
	cargo publish


#
# Testing
#
test:				test-unit test-integration
test-debug:			test-unit test-integration-debug
test-setup:			tests/node_modules

test-unit:
	RUST_BACKTRACE=1 cargo test

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
build-docs:
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
