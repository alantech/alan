SHELL = /bin/bash

.PHONY: build
build: env-check avm/target/release/alan build-js-runtime
	@echo Done

.PHONY: env-check
env-check:
	bash -c "./.envcheck.sh"

.PHONY: avm-style
avm-style:
	cd avm && cargo fmt -- --check

.PHONY: avm-unit
avm-unit: compiler/alan-compile
	cd avm && cargo test

.PHONY: compiler-browser-check
compiler-browser-check:
	cd compiler && yarn && yarn test

.PHONY: compiler-style
compiler-style:
	cd compiler && yarn && yarn style

./compiler/alan-compile:
	cd compiler && yarn
	yarn add pkg
	cd compiler && ../node_modules/.bin/pkg --targets host .

./avm/target/release/alan: compiler/alan-compile
	cd avm && cargo build --release
	cd avm && cargo fmt

.PHONY: build-js-runtime
build-js-runtime:
	cd js-runtime && yarn

.PHONY: bdd
bdd: shellspec node_modules
	bash -c "./bdd/bdd.sh $(testfile)"

shellspec:
	git clone --depth 1 --branch 0.27.2 https://github.com/shellspec/shellspec

node_modules: build
	npm init -y
	yarn add ./js-runtime

.PHONY: clean
clean:
	git clean -ffdxe .vscode

.PHONY: install
install: avm/target/release/alan
	cp ./avm/target/release/alan /usr/local/bin/alan

.PHONY: uninstall
uninstall:
	rm /usr/local/bin/alan

.PHONY: version
version:
	./.version.sh $(version)

.PHONY: prerelease
prerelease:
	./.prerelease.sh $(version)