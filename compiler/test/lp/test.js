const lp = require('../../dist/lp')

const helloWorldFile = new lp.LP('./hello_world.txt')

console.log(helloWorldFile)

const hello = lp.Token.build('Hello')
const world = lp.Token.build('World')
const space = lp.Token.build(' ')
const punctuation = lp.Or.build([lp.Token.build(','), lp.Token.build('!')])

const anyspace = lp.ZeroOrMore.build(space)
const helloWorld = lp.And.build([hello, punctuation, anyspace, world, punctuation, anyspace])

const result = helloWorld.apply(helloWorldFile)

console.log(result)