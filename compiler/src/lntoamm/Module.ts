import * as Ast from './Ast'
import Constant from './Constant'
import Event from './Event'
import Operator from './Operator'
import Scope from './Scope'
import UserFunction from './UserFunction'
import { Fn, } from './Function'
import { FunctionType, Interface, OperatorType, Type, } from './Type'

const modules = {}

class Module {
  moduleScope: Scope
  exportScope: Scope

  constructor(rootScope: Scope) {
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
    path: string,
    ast: any, // ModuleContext
    rootScope: Scope,
    isStd: boolean = false,
  ) {
    // First, take the export scope of the root scope and put references to it in this module. If
    // it is a built-in std module, it inherits from the root scope, otherwise it attaches all
    // exported references. This way std modules get access to the opcode scope via inheritance and
    // 'normal' modules do not.
    let module = new Module(isStd ? rootScope : undefined)
    if (!isStd) {
      for (const rootModuleName of Object.keys(rootScope.vals)) {
        module.moduleScope.put(rootModuleName, rootScope.vals[rootModuleName])
      }
    }
    // Now, populate all of the imports
    const imports = ast.imports()
    for (const importAst of imports) {
      // Figure out which kind of import format we're dealing with
      const standardImport = importAst.standardImport()
      const fromImport = importAst.fromImport()
      // If it's a "standard" import, figure out what name to call it (if the user overrode it)
      // and then attach the entire module with that name to the local scope.
      if (!!standardImport) {
        let importName: string
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
        module.moduleScope.put(importName, importedModule.exportScope)
      }
      // If it's a "from" import, we're picking off pieces of the exported scope and inserting them
      // also potentially renaming them if requested by the user
      if (!!fromImport) {
        const importedModule = modules[Ast.resolveDependency(path, fromImport.dependency())]
        const vars = fromImport.varlist().renameablevar()
        for (const moduleVar of vars) {
          let importName: string
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
          if (thing instanceof Type && thing.iface) {
            const iface = thing.iface
            const typesToCheck = Object.keys(importedModule.exportScope.vals)
              .map(n => importedModule.exportScope.vals[n])
              .filter(v => v instanceof Type)
            const fnsToCheck = Object.keys(importedModule.exportScope.vals)
              .map(n => importedModule.exportScope.vals[n])
              .filter(v => v instanceof Array && v[0].microstatementInlining instanceof Function)
            const opsToCheck = Object.keys(importedModule.exportScope.vals)
              .map(n => importedModule.exportScope.vals[n])
              .filter(v => v instanceof Array && v[0] instanceof Operator)

            typesToCheck
              .filter(t => iface.typeApplies(t, importedModule.exportScope))
              .forEach(t => {
                module.moduleScope.put(t.typename, t)
              })

            fnsToCheck
              .filter(fn => {
                // TODO: Make this better and move it to the Interface file in the future
                return iface.functionTypes.some(
                  (ft: FunctionType) => ft.functionname === fn[0].getName()
                )
              })
              .forEach(fn => {
                module.moduleScope.put(fn[0].getName(), fn)
              })

            opsToCheck
              .filter(op => {
                return iface.operatorTypes.some(
                  (ot: OperatorType) => ot.operatorname === op[0].name
                )
              })
              .forEach(op => {
                module.moduleScope.put(op[0].name, op)
              })
          }
        }
      }
    }
    // Next, types
    const types = ast.types()
    for (const typeAst of types) {
      const newType = Type.fromAst(typeAst, module.moduleScope);
      module.moduleScope.put(newType.typename, newType.alias ? newType.alias : newType)
    }
    // Next, interfaces
    const interfaces = ast.interfaces()
    for (const interfaceAst of interfaces) {
      Interface.fromAst(interfaceAst, module.moduleScope);
      // Automatically inserts the interface into the module scope, we're done.
    }
    // Next, constants
    const constdeclarations = ast.constdeclaration()
    for (const constdeclaration of constdeclarations) {
      Constant.fromAst(constdeclaration, module.moduleScope)
    }
    // Next, events
    const events = ast.events()
    for (const eventAst of events) {
      const newEvent = Event.fromAst(eventAst, module.moduleScope)
      module.moduleScope.put(newEvent.name, newEvent)
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
        module.moduleScope.put(newFunc.getName(), [newFunc])
      } else {
        fns.push(newFunc)
      }
    }
    // Next, operators
    const operatorMapping = ast.operatormapping()
    for (const operatorAst of operatorMapping) {
      const isPrefix = operatorAst.INFIX() === null
      const name = operatorAst.fntoop().operators().getText().trim()
      const precedence = parseInt(operatorAst.opprecedence().NUMBERCONSTANT().getText(), 10)
      const fns = module.moduleScope.deepGet(operatorAst.fntoop().varn().getText()) as Array<Fn>
      if (fns == null) {
        console.error("Operator " + name + " declared for unknown function " + operatorAst.varn().getText())
        process.exit(-31)
      }
      const op = new Operator(
        name,
        precedence,
        isPrefix,
        fns,
      )
      const opsBox = module.moduleScope.deepGet(name) as Array<Operator>
      if (!opsBox) {
        module.moduleScope.put(name, [op])
      } else {
        // To make sure we don't accidentally mutate other scopes, we're cloning this operator list
        let ops = [...opsBox]
        ops.push(op)
        module.moduleScope.put(name, ops)
      }
    }
    // Next, exports, which can be most of the above
    const exports = ast.exports()
    for (const exportAst of exports) {
      if (exportAst.varn() != null) {
        const exportVar = module.moduleScope.deepGet(exportAst.varn().getText())
        const splitName = exportAst.varn().getText().split(".")
        module.moduleScope.put(splitName[splitName.length - 1], exportVar)
        module.exportScope.put(splitName[splitName.length - 1], exportVar)
      } else if (exportAst.types() != null) {
        const newType = Type.fromAst(exportAst.types(), module.moduleScope)
        const typeBox = !newType.alias ? newType : newType.alias
        module.moduleScope.put(newType.typename, typeBox)
        module.exportScope.put(newType.typename, typeBox)
      } else if (exportAst.interfaces() != null) {
        const interfaceBox = Interface.fromAst(exportAst.interfaces(), module.moduleScope)
        // Automatically inserts the interface into the module scope
        module.exportScope.put(interfaceBox.typename, interfaceBox)
      } else if (exportAst.constdeclaration() != null) {
        const constVal = Constant.fromAst(exportAst.constdeclaration(), module.moduleScope)
        module.exportScope.put(constVal.name, constVal)
      } else if (exportAst.functions() != null) {
        const newFunc = UserFunction.fromAst(exportAst.functions(), module.moduleScope)
        if (newFunc.getName() == null) {
          console.error("Module-level functions must have a name")
          process.exit(-19)
        }
        // Exported scope must be checked first because it will fall through to the not-exported
        // scope by default. Should probably create a `getShallow` for this case, but reordering
        // the two if blocks below is enough to fix things here.
        let expFns = module.exportScope.get(newFunc.getName()) as Array<Fn>
        if (!expFns) {
          module.exportScope.put(
            newFunc.getName(),
            [newFunc],
          )
        } else {
          expFns.push(newFunc)
        }
        let modFns = module.moduleScope.get(newFunc.getName()) as Array<Fn>
        if (!modFns) {
          module.moduleScope.put(
            newFunc.getName(),
            [newFunc],
          )
        } else {
          modFns.push(newFunc)
        }
      } else if (exportAst.operatormapping() != null) {
        const operatorAst = exportAst.operatormapping()
        const isPrefix = operatorAst.INFIX() == null
        const name = operatorAst.fntoop().operators().getText().trim()
        const precedence = parseInt(operatorAst.opprecedence().NUMBERCONSTANT().getText(), 10)
        let fns = module.exportScope.deepGet(operatorAst.fntoop().varn().getText()) as Array<Fn>
        if (!fns) {
          fns = module.moduleScope.deepGet(operatorAst.fntoop().varn().getText()) as Array<Fn>
          if (!!fns) {
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
          fns,
        )
        let modOpsBox = module.moduleScope.deepGet(name) as Array<Operator>
        if (!modOpsBox) {
          module.moduleScope.put(name, [op])
        } else {
          let ops = [...modOpsBox]
          ops.push(op)
          module.moduleScope.put(name, ops)
        }
        let expOpsBox = module.exportScope.deepGet(name) as Array<Operator>
        if (!expOpsBox) {
          module.exportScope.put(name, [op])
        } else {
          let ops = [...expOpsBox]
          ops.push(op)
          module.exportScope.put(name, ops)
        }
      } else if (exportAst.events() != null) {
        const newEvent = Event.fromAst(exportAst.events(), module.moduleScope)
        module.moduleScope.put(newEvent.name, newEvent)
        module.exportScope.put(newEvent.name, newEvent)
      } else {
        // What?
        console.error("What should be an impossible export state has been reached.")
        process.exit(-8)
      }
    }
    // Finally, event handlers, so they can depend on events that are exported from the same module
    const handlers = ast.handlers()
    for (const handlerAst of handlers) {
      let evt = null
      if (handlerAst.eventref().varn() != null) {
        evt = module.moduleScope.deepGet(handlerAst.eventref().varn().getText()) as Event
      } else if (handlerAst.eventref().calls() != null) {
        console.error("Not yet implemented!")
        process.exit(-19)
        // evt = AFunction.callFromAst(handlerAst.eventref().calls(), module.moduleScope)
      }
      if (!evt) {
        console.error("Could not find specified event: " + handlerAst.eventref().getText())
        process.exit(-20)
      }
      if (!(evt instanceof Event)) {
        console.error(handlerAst.eventref().getText() + " is not an event")
        process.exit(-21)
      }
      let fn = null
      if (handlerAst.varn() != null) {
        const fnName = handlerAst.varn().getText()
        const fns = module.moduleScope.deepGet(handlerAst.varn().getText()) as Array<Fn>
        if (!fns) {
          console.error("Could not find specified function: " + fnName)
          process.exit(-22)
        }
        if (!(fns instanceof Array && fns[0].microstatementInlining instanceof Function)) {
          console.error(fnName + " is not a function")
          process.exit(-23)
        }
        for (let i = 0; i < fns.length; i++) {
          if (evt.type.typename === "void" && Object.values(fns[i].getArguments()).length === 0) {
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
        (evt.type === Type.builtinTypes["void"] && Object.keys(fn.getArguments()).length !== 0)
      ) {
        console.error("Function provided for " + handlerAst.eventref().getText() + " has invalid argument signature")
        process.exit(-25)
      }
      evt.handlers.push(fn)
    }
    return module
  }

  static modulesFromAsts(
    astMap: object, // string to ModuleContext
    rootScope: Scope,
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

export default Module
