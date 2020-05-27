#!/usr/bin/env node

const fs = require('fs')
const path = require('path')


const outJson = {}
const stdDir = path.join(__dirname, '../../std')
const stdAsts = fs.readdirSync(stdDir).filter(n => /.ln$/.test(n)).forEach(n => {
  outJson[n] = fs.readFileSync(path.join(stdDir, n), { encoding: 'utf8', })
})

console.log(JSON.stringify(outJson))
