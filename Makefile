
SHELL		= bash


#
# Project
#
preview-crate:			test-debug
	cargo publish --dry-run
publish-crate:			test-debug
	CARGO_HOME=$(HOME)/.cargo cargo publish

use-local-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save-dev ../../node-holochain-backdrop
use-npm-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save-dev @whi/holochain-backdrop
use-local-client:
	cd tests; npm uninstall @whi/holochain-client
	cd tests; npm install --save-dev ../../holochain-client-js
use-npm-client:
	cd tests; npm uninstall @whi/holochain-client
	cd tests; npm install --save-dev @whi/holochain-client

use-local:		use-local-client use-local-backdrop
use-npm:		  use-npm-client   use-npm-backdrop


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

PRE_HDK_VERSION = "0.2.0-beta-rc.4"
NEW_HDK_VERSION = "0.3.0-beta-dev.2"

PRE_HH_VERSION = "0.2.0-beta-rc.3", features
NEW_HH_VERSION = "0.3.0-beta-dev.1", features

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' Cargo.toml tests/zomes/

update-hdk-version:
	git grep -l '$(PRE_HH_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HH_VERSION)|$(NEW_HH_VERSION)|g'
	git grep -l '$(PRE_HDK_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDK_VERSION)|$(NEW_HDK_VERSION)|g'
