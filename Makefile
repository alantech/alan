SHELL = /bin/bash

.PHONY: build
build: interpreter/build/libs/interpreter-1.0.jar build-compiler runtime/target/release/alan-runtime build-js-runtime
	echo Done


interpreter/build/libs/interpreter-1.0.jar: interpreter
	cd interpreter && ./gradlew build

interpreter:
	git clone git@github.com:alantech/interpreter

.PHONY: build-compiler
build-compiler: compiler
	cd compiler && yarn

compiler:
	git clone git@github.com:alantech/compiler
	cd compiler && git submodule init && git submodule update

runtime/target/release/alan-runtime: runtime
	cd runtime && cargo build --release

runtime:
	git clone git@github.com:alantech/runtime

.PHONY: build-js-runtime
build-js-runtime: js-runtime
	cd js-runtime && yarn

js-runtime:
	git clone git@github.com:alantech/js-runtime

.PHONY: bdd
bdd: shellspec build node_modules
	./bdd/bdd.sh

shellspec:
	git clone git@github.com:shellspec/shellspec

node_modules:
	yarn add ./compiler
	yarn add ./js-runtime

.PHONY: clean
clean:
	rm -rf interpreter
	rm -rf compiler
	rm -rf runtime
	rm -rf js-runtime
	rm -rf shellspec
	rm -rf node_modules
	rm -rf bdd/node_modules
	rm -f package.json
	rm -f yarn.lock
	rm -f bdd/package-lock.json

.PHONY: install
install: runtime/target/release/alan-runtime interpreter/build/libs/interpreter-1.0.jar interpreter compiler
	cp ./alan /usr/local/bin/alan
	cp ./runtime/target/release/alan-runtime /usr/local/bin/alan-runtime
	mkdir -p /usr/local/bin/build/libs # TODO: Remove when interpreter dies
	cp ./interpreter/build/libs/interpreter-1.0.jar /usr/local/bin/build/libs/interpreter-1.0.jar
	cp ./interpreter/alan-interpreter /usr/local/bin/alan-interpreter
	# npm install -g ./compiler

.PHONY: uninstall
uninstall:
	rm -rf /usr/local/bin/build
	rm /usr/local/bin/alan
	rm /usr/local/bin/alan-runtime
	rm /usr/local/bin/alan-interpreter
	# npm uninstall -g alan-compile