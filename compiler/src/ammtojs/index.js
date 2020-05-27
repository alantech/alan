const Ast = require('../amm/Ast')
const AsyncOpcodes = require('alan-js-runtime').asyncopcodes

const callToJsText = (call) => {
  const args = call.calllist() ? call.calllist().VARNAME().map(v => v.getText()).join(', ') : ""
  const opcode = call.VARNAME().getText()
  return AsyncOpcodes.includes(opcode) ? `await r.${opcode}(${args})` : `r.${opcode}(${args})`
}

const functionbodyToJsText = (fnbody, indent) => {
  let outText = ""
  for (const statement of fnbody.statements()) {
    outText += indent + "  " // For legibility of the output
    if (statement.declarations()) {
      if (statement.declarations().constdeclaration()) {
        const dec = statement.declarations().constdeclaration()
        outText += `const ${dec.decname().getText()} = ${assignableToJsText(dec.assignables(), indent)}\n`
      } else if (statement.declarations().letdeclaration()) {
        const dec = statement.declarations().letdeclaration()
        outText += `let ${dec.decname().getText()} = ${assignableToJsText(dec.assignables(), indent)}\n`
      }
    } else if (statement.assignments()) {
      const assign = statement.assignments()
      outText += `${assign.decname().getText()} = ${assignableToJsText(assign.assignables(), indent)}\n`
    } else if (statement.calls()) {
      outText += `${callToJsText(statement.calls())}\n`
    } else if (statement.emits()) {
      const emit = statement.emits()
      const name = emit.VARNAME(0).getText()
      const arg = emit.VARNAME(1) ? emit.VARNAME(1).getText() : 'undefined'
      outText += `r.emit('${name}', ${arg})\n`
    }
  }
  return outText
}

const assignableToJsText = (assignable, indent) => {
  let outText = ""
  if (assignable.functions()) {
    const fn = assignable.functions()
    outText += '() => {\n' // All assignable functions/closures take no arguments
    outText += functionbodyToJsText(assignable.functions().functionbody(), indent + "  ")
    outText += indent + '  }' // End this closure
  } else if (assignable.calls()) {
    outText += callToJsText(assignable.calls())
  } else if (assignable.VARNAME()) {
    outText += assignable.VARNAME().getText()
  } else if (assignable.constants()) {
    outText += assignable.constants().getText()
  } else if (assignable.objectliterals()) {
    // TODO: Actually do this right once we figure out what we even want to do with object literals
    throw new Error('Object literals not yet implemented!')
  }
  return outText
}

const ammToJsText = (amm) => {
  let outFile = "const r = require('alan-js-runtime')\n"
  // Where we're going we don't need types, so skipping that entire section
  // First convert all of the global constants to javascript
  for (const globalConst of amm.constdeclaration()) {
    outFile +=
      `const ${globalConst.decname().getText()} = ${assignableToJsText(globalConst.assignables(), '')}\n`
  }
  // We can also skip the event declarations because they are lazily bound by EventEmitter
  // Now we convert the handlers to Javascript. This is the vast majority of the work
  for (const handler of amm.handlers()) {
    const eventVarName = handler.functions().VARNAME() ? handler.functions().VARNAME().getText() : ""
    outFile += `r.on('${handler.VARNAME().getText()}', async (${eventVarName}) => {\n`
    outFile += functionbodyToJsText(handler.functions().functionbody(), '')
    outFile += '})\n' // End this handler
  }
  outFile += "r.emit('_start', undefined)\n" // Let's get it started in here
  return outFile
}

module.exports = (filename) => ammToJsText(Ast.fromFile(filename))
module.exports.ammTextToJs = (str) => ammToJsText(Ast.fromString(str))
