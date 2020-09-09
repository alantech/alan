SHELL = /bin/bash

.PHONY: build
build: env-check build-compiler runtime/target/release/alan build-js-runtime
	echo Done

.PHONY: env-check
env-check:
	bash -c "./.envcheck.sh"

.PHONY: runtime-unit
runtime-unit: compiler/alan-compile
	cd runtime && cargo test

.PHONY: compiler-browser-check
compiler-browser-check: build-compiler
	cd compiler && yarn test

.PHONY: build-compiler
build-compiler:
	cd compiler && yarn

compiler/alan-compile: build-compiler
	yarn add nexe
	cd compiler && ../node_modules/.bin/nexe -r std -o alan-compile

runtime/target/release/alan: compiler/alan-compile
	cd runtime && cargo build --release

.PHONY: build-js-runtime
build-js-runtime:
	cd js-runtime && yarn

.PHONY: bdd
bdd: shellspec node_modules
	bash -c "./bdd/bdd.sh $(testfile)"

shellspec:
	git clone --depth 1 https://github.com/shellspec/shellspec

node_modules: build
	npm init -y
	yarn add ./compiler
	yarn add ./js-runtime
	cp -r ./js-runtime/* ./node_modules/alan-compile/node_modules/alan-js-runtime
	cp -r ./js-runtime/* ./compiler/node_modules/alan-js-runtime

.PHONY: clean
clean:
	git clean -ffdx

.PHONY: install
install: runtime/target/release/alan
	cp ./runtime/target/release/alan /usr/local/bin/alan

.PHONY: uninstall
uninstall:
	rm /usr/local/bin/alan