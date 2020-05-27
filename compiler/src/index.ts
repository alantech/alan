#!/usr/bin/env node

const fs = require('fs')

const commander = require('commander')

const ammtoagc = require('./ammtoagc')
const ammtojs = require('./ammtojs')
const lntoagc = require('./lntoagc')
const lntoamm = require('./lntoamm')
const lntojs = require('./lntojs')
const package = require('../package.json')

const getFormat = (filename: string) => filename.replace(/^.+\.([A-Za-z0-9]{2,3})$/g, "$1")

const convert = {
  ln: {
    agc: lntoagc,
    amm: lntoamm,
    js: lntojs,
  },
  amm: {
    agc: ammtoagc,
    js: ammtojs,
  }
}

let inputfile: string, outputfile: string
commander
  .name(package.name)
  .version(package.version)
  .arguments('<input> <output>')
  .action((input: string, output:string ) => {
    inputfile = input
    outputfile = output
  })
  .description('Compile the specified source file to the specified output file')
  .parse(process.argv)

if (convert[getFormat(inputfile)] && convert[getFormat(inputfile)][getFormat(outputfile)]) {
  const output = convert[getFormat(inputfile)][getFormat(outputfile)](inputfile)
  fs.writeFileSync(outputfile, output, { encoding: 'utf8', })
  console.log('Done!')
} else {
  console.error(`${getFormat(inputfile)} to ${getFormat(outputfile)} not implemented!`)
  process.exit(1)
}

