SHELL = /bin/bash

.PHONY: build
build: env-check build-compiler runtime/target/release/alan-runtime build-js-runtime
	echo Done

.PHONY: env-check
env-check:
	./.envcheck.sh

.PHONY: runtime-unit
runtime-unit:
	cd runtime && cargo test

.PHONY: compiler-browser-check
compiler-browser-check: build-compiler
	cd compiler && yarn test

.PHONY: build-compiler
build-compiler:
	cd compiler && yarn

runtime/target/release/alan-runtime: runtime
	cd runtime && cargo build --release

.PHONY: build-js-runtime
build-js-runtime:
	cd js-runtime && yarn

.PHONY: bdd
bdd: shellspec node_modules
	./bdd/bdd.sh $(testfile)

shellspec:
	git clone --depth 1 git@github.com:shellspec/shellspec

node_modules: build
	npm init -y
	yarn add ./compiler
	yarn add ./js-runtime

.PHONY: clean
clean:
	git clean -ffdx

.PHONY: install
install: runtime/target/release/alan-runtime node_modules
	cp ./runtime/target/release/alan-runtime /usr/local/bin/alan-runtime
	npm install -g ./compiler

.PHONY: uninstall
uninstall:
	rm /usr/local/bin/alan-runtime
	npm uninstall -g alan-compile