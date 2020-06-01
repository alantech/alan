const lp = require('../../dist/lp')
const start = Date.now()
const helloWorldFile = new lp.LP('./hello_world.txt')
const newLP = Date.now()
console.log(`File load and constructor time: ${newLP - start}ms`)
console.log(helloWorldFile)

const hello = lp.Token.build('Hello')
const world = lp.Token.build('World')
const space = lp.Token.build(' ')
const punctuation = lp.Or.build([lp.Token.build(','), lp.Token.build('!')])

const anyspace = lp.ZeroOrMore.build(space)
const helloWorld = lp.NamedAnd.build({
  hello,
  a: punctuation,
  b: anyspace,
  world,
  c: punctuation,
  d: anyspace
})
const grammar = Date.now()
console.log(`Grammar definition time: ${grammar - newLP}ms`)
const result = helloWorld.apply(helloWorldFile)
const calc = Date.now()
console.log(result)
console.log(`Parsing time: ${calc - grammar}ms`)