#!/usr/bin/env node
Error.stackTraceLimit = Infinity

import * as fs from 'fs';

import commander = require('commander');

import buildPipeline from './pipeline';
import * as agatoagc from './agatoagc';
import * as agctoagz from './agctoagz';
import * as ammtoaga from './ammtoaga';
import * as ammtojs from './ammtojs';
import * as lntoamm from './lntoamm';
import * as lnntoamm from './lnntoamm';

const start = Date.now();

const getFormat = (filename: string) =>
  filename.replace(/^.+\.([A-Za-z0-9]{2,3})$/g, '$1');

const formatTime = (ms: number) => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${ms / 1000.0}s`;
  const minutes = Math.floor(ms / 60000);
  const remaining = ms - minutes * 60000;
  return `${minutes}min ${remaining / 1000.0}s`;
};

const convert = buildPipeline([
  ['ln', 'amm', lntoamm],
  ['lnn', 'amm', lnntoamm],
  ['amm', 'aga', ammtoaga],
  ['amm', 'js', ammtojs],
  ['aga', 'agc', agatoagc],
  ['agc', 'agz', agctoagz],
]);

let inputfile: string, outputfile: string;
commander
  .name('alan-compile')
  .version('0.1.0') // TODO: Try to revive getting this from package.json; it's just weird in TS
  .arguments('<input> <output>')
  .action((input: string, output: string) => {
    inputfile = input;
    outputfile = output;
  })
  .description(
    `Compile the specified source file to the specified output file

The input and output formats are determined automatically by the file extensions specified

> alan-compile myRootSourceFile.ln myApplication.agc

The AGC format is used by the alan-runtime as its native binary format

> alan-compile mySourceFile.ln myWebApplication.js

The compiler can also transpile to JS for use in Node.js or the browser

It is also possible to get the compiler's intermediate representations, AMM and AGA:

> alan-compile mySourceFile.ln firstIntermediateLayer.amm
> alan-compile firstIntermediateLayer.amm secondIntermediateLayer.aga

And to resume from these intermediate representations

> alan-compile firstIntermediateLayer.amm myWebApplication.js
> alan-compile secondIntermediateLayer.aga myApplication.agc

Supports the following input formats:
- ln (Alan source code)
- lnn (Alan source code, using the new compiler front-end)
- amm (Alan-- intermediate representation)
- aga (Alan Graphcode Assembler representation)

Supports the following output formats
- amm (Alan-- intermediate representation)
- js (Transpilation to Javascript)
- aga (Alan Graphcode Assembler representation)
- agc (Compilation to Alan Graphcode format used by the alan-runtime)
`,
  )
  .parse(process.argv);

if (
  convert[getFormat(inputfile)] &&
  convert[getFormat(inputfile)][getFormat(outputfile)]
) {
  try {
    const output =
      convert[getFormat(inputfile)][getFormat(outputfile)].fromFile(inputfile);
    fs.writeFileSync(outputfile, output, { encoding: 'utf8' });
    const end = Date.now();
    console.log(`Done in ${formatTime(end - start)}`);
  } catch (e) {
    console.error(e);
    process.exit(1);
  }
} else {
  console.error(
    `${getFormat(inputfile)} to ${getFormat(outputfile)} not implemented!`,
  );
  process.exit(2);
}
