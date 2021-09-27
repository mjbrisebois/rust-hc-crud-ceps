
SHELL		= bash


#
# Project
#
tests/package-lock.json:	tests/package.json
	touch $@
tests/node_modules:		tests/package-lock.json
	cd tests; \
	npm install
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

test-unit:
	RUST_BACKTRACE=1 cargo test

# test-integration:		integration
# 	cd tests; RUST_LOG=none npx mocha integration/test_basic.js
# test-integration-debug:		integration
# 	cd tests; RUST_LOG=info LOG_LEVEL=silly npx mocha integration/test_basic.js


#
# Documentation
#
build-docs:			build-mere-memory-docs
build-mere-memory-docs:
	cd zomes; cargo doc -p hc_zome_mere_memory


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
