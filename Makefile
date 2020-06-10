SHELL = /bin/bash

.PHONY: build
build: build-compiler runtime/target/release/alan-runtime build-js-runtime
	echo Done

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
	yarn add ./compiler
	yarn add ./js-runtime

.PHONY: clean
clean:
	cd runtime && cargo clean
	cd compiler && yarn clean
	rm -rf shellspec
	rm -rf node_modules
	rm -rf compiler/node_modules
	rm -rf js-runtime/node_modules
	rm -rf bdd/node_modules
	rm -rf compiler/std
	rm -f package.json
	rm -f yarn.lock
	rm -f bdd/package-lock.json
	rm -f bdd/temp.*

.PHONY: install
install: runtime/target/release/alan-runtime node_modules
	cp ./alan /usr/local/bin/alan
	cp ./runtime/target/release/alan-runtime /usr/local/bin/alan-runtime
	npm install -g ./compiler

.PHONY: uninstall
uninstall:
	rm -rf /usr/local/bin/build
	rm /usr/local/bin/alan
	rm /usr/local/bin/alan-runtime
	npm uninstall -g alan-compile