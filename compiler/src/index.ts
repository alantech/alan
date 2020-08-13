#!/usr/bin/env node

import * as fs from 'fs'

import commander = require('commander')

import buildPipeline from './pipeline'
import * as ammtoaga from './ammtoaga'
import * as agatoagc from './agatoagc'
import * as ammtojs from './ammtojs'
import * as lntoamm from './lntoamm'

const start = Date.now()

const getFormat = (filename: string) => filename.replace(/^.+\.([A-Za-z0-9]{2,3})$/g, "$1")

const formatTime = (ms: number) => {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${ms / 1000.0}s`
  const minutes = Math.floor(ms / 60000)
  const remaining = ms - (minutes * 60000)
  return `${minutes}min ${remaining / 1000.0}s`
}

const convert = buildPipeline([
  ['ln', 'amm', lntoamm],
  ['amm', 'aga', ammtoaga],
  ['amm', 'js', ammtojs],
  ['aga', 'agc', agatoagc],
])

let inputfile: string, outputfile: string
commander
  .name('alan-compiler')
  .version('0.1.0') // TODO: Try to revive getting this from package.json; it's just weird in TS
  .arguments('<input> <output>')
  .action((input: string, output:string ) => {
    inputfile = input
    outputfile = output
  })
  .description('Compile the specified source file to the specified output file')
  .parse(process.argv)

if (convert[getFormat(inputfile)] && convert[getFormat(inputfile)][getFormat(outputfile)]) {
  try {
    const output = convert[getFormat(inputfile)][getFormat(outputfile)].fromFile(inputfile)
    fs.writeFileSync(outputfile, output, { encoding: 'utf8', })
    const end = Date.now()
    console.log(`Done in ${formatTime(end - start)}`)
  } catch (e) {
    console.error(e.message)
    process.exit(1)
  }
} else {
  console.error(`${getFormat(inputfile)} to ${getFormat(outputfile)} not implemented!`)
  process.exit(2)
}

