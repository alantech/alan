# alan-compile

A compiler for alan to Javascript and Alan Graphcode, the [runtime](https://github.com/alantech/alan/tree/master/runtime)'s bytecode format.

Caveats: There should be a strict mode that make sure int64s are using [BigInt](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt) despite the performance loss, but the vast majority of the time falling back to regular numbers should work.

This compiler is licensed AGPL 3.0 but the [alan standard library](https://github.com/alantech/alan/tree/master/std) and the [Javascript runtime shim](https://github.com/alantech/alan/tree/master/js-runtime) are licensed Apache 2.0 so you can freely distribute your compiled code.

## Install

```sh
npm install -g alan-compile
```

## Usage

```sh
alan-compile <sourcefile> <outputfile>
```

The compiler uses the file extension to determine what compilation steps to perform.

The supported source formats are:

* `.ln` - The a*l*a*n* source file extension, with automatic traversal and loading of specified imports relative to this file.
* `.amm` - The *a*lan *m*inus *m*inus (`alan--`) intermediate representation. A strict subset of `alan` in a single file used for final conversion to the output formats.
* `.aga` - The *a*lan *g*raph *a*ssembler format. An intermediate representation very close to the `.agc` format (below) that the runtime operates on. It also indicates the dependency graph of operations in the format that can be used by the runtime. Useful for debugging runtime behavior issues or if you are targeting the runtime with a different language.

The supported output formats are:

* `.amm` - The *a*lan *m*inus *m*inus (`alan--`) intermediate representation, useful only for debugging compiler issues or if you are writing your own second stage compiler for another runtime environment.
* `.aga` - The *a*lan *g*raph *a*ssembler format. An intermediate representation very close to the `.agc` format (below) that the runtime operates on. It also indicates the dependency graph of operations in the format that can be used by the runtime. Useful for debugging runtime behavior issues or if you are targeting the runtime with a different language.
* `.agc` - The *a*lan *g*raph*c*ode bytecode format. The bytecode format of the alan [runtime](https://github.com/alantech/runtime) that also maintains a dependency graph of operations to allow quick, dynamic restructuring of the code safely depending on the data being processed and the state and capabilities of the machine it is running on.
* `.js` - The most common [ECMAScript](https://ecma-international.org/ecma-262/10.0/index.html) file extension, representing a [CommonJS](http://www.commonjs.org/) module (aka a [Node](https://nodejs.org/en/) module).

Note: `.amm` to `.amm` is absurd and not supported. :)

## Browser Support

This project also uses [Browserify](http://browserify.org/) to create a version of the compiler that works directly in the browser. The browser version of the compiler does not support output to `.agc`, but does support `.js` which can be simply `eval()`ed to execute.

To get this bundled browser version, simply run:

```sh
yarn bundle
```

and copy the resulting `bundle.js` to your own project, include it in a `<script>` tag:

```html
<script src="bundle.js"></script>
```

then in your own Javascript source included later, you can acquire and use the compiler in this way:

```js
const alanCompile = require('alan-compile') // Browserify creates a toplevel `require` function that you can use to get the modules
const helloWorld = alanCompile('ln', 'js', `
  import @std/app

  on app.start {
    app.print("Hello, World!")
    emit app.exit 0
  }
`) // argument order is: sourceExtension, outputExtension, sourceCode
eval(helloWorld) // Execute the generated javascript code
```

### Licensing Warning

While the Alan Standard Library and Alan JS Runtime are Apache 2.0 license and therefore freely distributable with your own code, the Alan Compiler is AGPL 3.0 licensed, so embedding the compiler in this way requires the project using it to also be AGPL 3.0 licensed.

Pregenerating the Javascript to run in your own build system from the CLI tool does not cause this licensing escalation. However, this is only a problem in a minority of use-cases as the Javascript source is already transmitted to the users.

## Development

`alan-compile` is written in relatively standard Node.js+Typescript with the source code in `src`. Run `yarn build` to recompile (or just `tsc` if you have that installed globally).

### Contribution Agreement

To contribute to `alan-compile` you need to sign a Contributor License Agreement, Alan Technologies will retain the right to relicense this code in licenses other than AGPL 3.0 concurrently or in the future to convert to a newer license.

## License

AGPL 3.0
