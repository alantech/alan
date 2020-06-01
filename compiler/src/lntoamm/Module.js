const Ast = require('./Ast')
const Box = require('./Box')
const Event = require('./Event')
const Interface = require('./Interface')
const Operator = require('./Operator')
const Scope = require('./Scope')
const UserFunction = require('./UserFunction')
const Type = require('./Type')

const modules = {}

class Module {
  constructor(rootScope) {
    // Thoughts on how to handle this right now:
    // 1. The outermost module scope is read-only always.
    // 2. Therefore anything in the export scope can simply be duplicated in both scopes
    // 3. Therefore export scope needs access to the module scope so the functions function, but
    //    the module scope can just use its local copy
    this.moduleScope = new Scope(rootScope)
    this.exportScope = new Scope(this.moduleScope)
  }

  static getAllModules() {
    return modules
  }
  
  static populateModule(
    path, // string
    ast, // ModuleContext
    rootScope // Scope
  ) {
    let module = new Module(rootScope)
    // First, populate all of the imports
    const imports = ast.imports()
    for (const importAst of imports) {
      // Figure out which kind of import format we're dealing with
      const standardImport = importAst.standardImport()
      const fromImport = importAst.fromImport()
      // If it's a "standard" import, figure out what name to call it (if the user overrode it)
      // and then attach the entire module with that name to the local scope.
      if (!!standardImport) {
        let importName
        if (standardImport.AS() != null) {
          importName = standardImport.VARNAME().getText()
        } else if (standardImport.dependency().localdependency() != null) {
          let nameParts = standardImport.dependency().localdependency().getText().split("/")
          importName = nameParts[nameParts.length - 1]
        } else if (standardImport.dependency().globaldependency() != null) {
          let nameParts = standardImport.dependency().globaldependency().getText().split("/")
          importName = nameParts[nameParts.length - 1]
        } else {
          // What?
          console.error("This path should be impossible")
          process.exit(-3)
        }
        const importedModule = modules[Ast.resolveDependency(path, standardImport.dependency())]
        module.moduleScope.put(importName, new Box(importedModule.exportScope))
      }
      // If it's a "from" import, we're picking off pieces of the exported scope and inserting them
      // also potentially renaming them if requested by the user
      if (!!fromImport) {
        const importedModule = modules[Ast.resolveDependency(path, fromImport.dependency())]
        const vars = fromImport.varlist().renameablevar()
        for (const moduleVar of vars) {
          let importName
          const exportName = moduleVar.varop(0).getText()
          if (moduleVar.AS() != null) {
            importName = moduleVar.varop(1).getText()
          } else {
            importName = moduleVar.varop(0).getText()
          }
          const thing = importedModule.exportScope.get(exportName)
          module.moduleScope.put(importName, thing)
          // Special behavior for interfaces. If there are any functions or operators that match
          // the interface, pull them in. Similarly any types that match the entire interface. This
          // allows concise importing of a related suite of tools without having to explicitly call
          // out each one.
          if (thing.typeval && thing.typeval.iface) {
            const iface = thing.typeval.iface
            const typesToCheck = Object.keys(importedModule.exportScope.vals)
              .map(n => importedModule.exportScope.vals[n])
              .filter(v => !!v.typeval)
            const fnsToCheck = Object.keys(importedModule.exportScope.vals)
              .map(n => importedModule.exportScope.vals[n])
              .filter(v => !!v.functionval)
            /**
             * Add opsToCheck after the code to declare operators in interfaces is added
            const opsToCheck = Object.keys(importedModule.exportScope)
              .map(n => importedModule.exportScope[n])
              .filter(v => !!v.operatorval)
             */

            typesToCheck
              .filter(t => iface.typeApplies(t.typeval, importedModule.exportScope))
              .forEach(t => {
                module.moduleScope.put(t.typeval.typename, t)
              })

            fnsToCheck
              .filter(fn => {
                // TODO: Make this better and move it to the Interface file in the future
                return iface.functionTypes.some(
                  ft => ft.functionname === fn.functionval[0].getName()
                )
              })
              .forEach(fn => {
                module.moduleScope.put(fn.functionval[0].getName(), fn)
              })
          }
        }
      }
    }
    // Next, types
    const types = ast.types()
    for (const typeAst of types) {
      const newType = Type.fromAst(typeAst, module.moduleScope);
      module.moduleScope.put(newType.typename, new Box(
        newType.alias === null ? newType : newType.alias
      ))
    }
    // Next, interfaces
    const interfaces = ast.interfaces()
    for (const interfaceAst of interfaces) {
      Interface.fromAst(interfaceAst, module.moduleScope);
      // Automatically inserts the interface into the module scope, we're done.
    }
    // Next, constants
    const constdeclarations = ast.constdeclaration()
    for (const constAst of constdeclarations) {
      const newConst = Box.fromConstAst(constAst, module.moduleScope)
      let constName
      if (constAst.VARNAME() != null) {
        constName = constAst.VARNAME().getText()
      } else {
        constName = constAst.assignments().varn().getText()
      }
      module.moduleScope.put(constName, newConst)
    }
    // Next, events
    const events = ast.events()
    for (const eventAst of events) {
      const newEvent = Event.fromAst(eventAst, module.moduleScope)
      module.moduleScope.put(newEvent.name, new Box(newEvent, true))
    }
    // Next, functions
    const functions = ast.functions()
    for (const functionAst of functions) {
      const newFunc = UserFunction.fromAst(functionAst, module.moduleScope)
      if (newFunc.getName() == null) {
        console.error("Module-level functions must have a name")
        process.exit(-19)
      }
      let fns = module.moduleScope.get(newFunc.getName())
      if (fns == null) {
        module.moduleScope.put(newFunc.getName(), new Box([newFunc], true))
      } else {
        fns.functionval.push(newFunc)
      }
    }
    // Next, operators
    const operatorMapping = ast.operatormapping()
    for (const operatorAst of operatorMapping) {
      const isPrefix = operatorAst.infix() == null
      let isCommutative = false
      let isAssociative = false
      if (!isPrefix) {
        isCommutative = operatorAst.infix().COMMUTATIVE() != null
        isAssociative = operatorAst.infix().ASSOCIATIVE() != null
      }
      const name = operatorAst.operators().getText().trim()
      const precedence = parseInt(operatorAst.NUMBERCONSTANT().getText(), 10)
      const fns = module.moduleScope.deepGet(operatorAst.varn())
      if (fns == null) {
        console.error("Operator " + name + " declared for unknown function " + operatorAst.varn().getText())
        process.exit(-31)
      }
      const op = new Operator(
        name,
        precedence,
        isPrefix,
        isCommutative,
        isAssociative,
        fns.functionval,
      )
      const opsBox = module.moduleScope.deepGet(name)
      if (opsBox == null) {
        module.moduleScope.put(name, new Box([op]))
      } else {
        // To make sure we don't accidentally mutate other scopes, we're cloning this operator list
        let ops = [...opsBox.operatorval]
        ops.push(op)
        module.moduleScope.put(name, new Box(ops))
      }
    }
    // Next, exports, which can be most of the above
    const exports = ast.exports()
    for (const exportAst of exports) {
      if (exportAst.varn() != null) {
        const exportVar = module.moduleScope.deepGet(exportAst.varn())
        const splitName = exportAst.varn().getText().split(".")
        module.moduleScope.put(splitName[splitName.length - 1], exportVar)
        module.exportScope.put(splitName[splitName.length - 1], exportVar)
      } else if (exportAst.types() != null) {
        const newType = Type.fromAst(exportAst.types(), module.moduleScope)
        const typeBox = new Box(!newType.alias ? newType : newType.alias)
        module.moduleScope.put(newType.typename, typeBox)
        module.exportScope.put(newType.typename, typeBox)
      } else if (exportAst.interfaces() != null) {
        const interfaceBox = Interface.fromAst(exportAst.interfaces(), module.moduleScope)
        // Automatically inserts the interface into the module scope
        module.exportScope.put(interfaceBox.typeval.typename, interfaceBox)
      } else if (exportAst.constdeclaration() != null) {
        const newConst = Box.fromConstAst(exportAst.constdeclaration(), module.moduleScope)
        let constName
        if (exportAst.constdeclaration().VARNAME() != null) {
          constName = exportAst.constdeclaration().VARNAME().getText()
        } else {
          constName = exportAst.constdeclaration().assignments().varn().getText()
        }
        module.moduleScope.put(constName, newConst)
        module.exportScope.put(constName, newConst)
      } else if (exportAst.functions() != null) {
        const newFunc = UserFunction.fromAst(exportAst.functions(), module.moduleScope)
        if (newFunc.getName() == null) {
          console.error("Module-level functions must have a name")
          process.exit(-19)
        }
        // Exported scope must be checked first because it will fall through to the not-exported
        // scope by default. Should probably create a `getShallow` for this case, but reordering
        // the two if blocks below is enough to fix things here.
        let expFns = module.exportScope.get(newFunc.getName())
        if (expFns == null) {
          module.exportScope.put(newFunc.getName(), new Box([newFunc], true))
        } else {
          expFns.functionval.push(newFunc)
        }
        let modFns = module.moduleScope.get(newFunc.getName())
        if (modFns == null) {
          module.moduleScope.put(newFunc.getName(), new Box([newFunc], true))
        } else {
          modFns.functionval.push(newFunc)
        }
      } else if (exportAst.operatormapping() != null) {
        const operatorAst = exportAst.operatormapping()
        const isPrefix = operatorAst.infix() == null
        let isCommutative = false
        let isAssociative = false
        if (!isPrefix) {
          isCommutative = operatorAst.infix().COMMUTATIVE() != null
          isAssociative = operatorAst.infix().ASSOCIATIVE() != null
        }
        const name = operatorAst.operators().getText().trim();
        const precedence = parseInt(operatorAst.NUMBERCONSTANT().getText(), 10)
        let fns = module.exportScope.deepGet(operatorAst.varn())
        if (fns == null) {
          fns = module.moduleScope.deepGet(operatorAst.varn())
          if (fns != null) {
            console.error(
              "Exported operator " +
              name +
              " wrapping unexported function " +
              operatorAst.varn().getText() +
              " which is not allowed, please export the function, as well."
            )
            process.exit(-32)
          }
          console.error("Operator " + name + " declared for unknown function " + operatorAst.varn().getText())
          process.exit(-33)
        }
        const op = new Operator(
          name,
          precedence,
          isPrefix,
          isCommutative,
          isAssociative,
          fns.functionval,
        )
        let modOpsBox = module.moduleScope.deepGet(name)
        if (modOpsBox == null) {
          module.moduleScope.put(name, new Box([op]))
        } else {
          let ops = [...modOpsBox.operatorval]
          ops.push(op)
          module.moduleScope.put(name, new Box(ops))
        }
        let expOpsBox = module.exportScope.deepGet(name)
        if (expOpsBox == null) {
          module.exportScope.put(name, new Box([op]))
        } else {
          let ops = [...expOpsBox.operatorval]
          ops.push(op)
          module.exportScope.put(name, new Box(ops))
        }
      } else if (exportAst.events() != null) {
        const newEvent = Event.fromAst(exportAst.events(), module.moduleScope)
        module.moduleScope.put(newEvent.name, new Box(newEvent, true))
        module.exportScope.put(newEvent.name, new Box(newEvent, true))
      } else {
        // What?
        console.error("What should be an impossible export state has been reached.")
        process.exit(-8)
      }
    }
    // Finally, event handlers, so they can depend on events that are exported from the same module
    const handlers = ast.handlers()
    for (const handlerAst of handlers) {
      let eventBox = null
      if (handlerAst.eventref().varn() != null) {
        const eventName = handlerAst.eventref().varn().getText()
        eventBox = module.moduleScope.deepGet(handlerAst.eventref().varn())
      } else if (handlerAst.eventref().calls() != null) {
        eventBox = AFunction.callFromAst(handlerAst.eventref().calls(), module.moduleScope)
      }
      if (eventBox == null) {
        console.error("Could not find specified event: " + handlerAst.eventref().getText())
        process.exit(-20)
      }
      if (eventBox.type != Box.builtinTypes["Event"]) {
        console.error(eventBox)
        console.error(handlerAst.eventref().getText() + " is not an event")
        process.exit(-21)
      }
      const evt = eventBox.eventval
      let fn = null
      if (handlerAst.varn() != null) {
        const fnName = handlerAst.varn().getText()
        const fnBox = module.moduleScope.deepGet(handlerAst.varn())
        if (fnBox == null) {
          console.error("Could not find specified function: " + fnName)
          process.exit(-22)
        }
        if (fnBox.type != Box.builtinTypes["function"]) {
          console.error(fnName + " is not a function")
          process.exit(-23)
        }
        const fns = fnBox.functionval
        for (let i = 0; i < fns.length; i++) {
          if (evt.type.typename === "void" && fns[i].getArguments().values().size() === 0) {
            fn = fns[i]
            break
          }
          const argTypes = Object.values(fns[i].getArguments())
          if (argTypes.length !== 1) continue
          if (argTypes[0] == evt.type) {
            fn = fns[i]
            break
          }
        }
        if (fn == null) {
          console.error("Could not find function named " + fnName + " with matching function signature")
          process.exit(-35)
        }
      }
      if (handlerAst.functions() != null) {
        fn = UserFunction.fromAst(handlerAst.functions(), module.moduleScope)
      }
      if (handlerAst.functionbody() != null) {
        fn = UserFunction.fromAst(handlerAst.functionbody(), module.moduleScope)
      }
      if (fn == null) {
        // Shouldn't be possible
        console.error("Impossible state reached processing event handler")
        process.exit(-24)
      }
      if (Object.keys(fn.getArguments()).length > 1 ||
        (evt.type == Box.builtinTypes["void"] && Object.keys(fn.getArguments()).length != 0)
      ) {
        console.error("Function provided for " + handlerAst.eventref().getText() + " has invalid argument signature")
        process.exit(-25)
      }
      evt.handlers.push(fn)
    }
    return module
  }

  static modulesFromAsts(
    astMap, // string to ModuleContext
    rootScope // Scope
  ) {
    let modulePaths = Object.keys(astMap)
    while (modulePaths.length > 0) {
      for (let i = 0; i < modulePaths.length; i++) {
        const path = modulePaths[i]
        const moduleAst = astMap[path]
        const imports = Ast.resolveImports(path, moduleAst)
        let loadable = true
        for (const importPath of imports) {
          if (importPath[0] === '@') continue
          if (modules.hasOwnProperty(importPath)) continue
          loadable = false
        }
        if (!loadable) continue
        modulePaths.splice(i, 1)
        i--
        const module = Module.populateModule(path, moduleAst, rootScope)
        modules[path] = module
      }
    }
    return modules
  }
}

module.exports = Module
