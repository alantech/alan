import * as Ast from './Ast';
import Constant from './Constant';
import Event from './Event';
import Operator from './Operator';
import Scope from './Scope';
import UserFunction from './UserFunction';
import { Fn } from './Function';
import { FunctionType, Interface, OperatorType, Type } from './Type';
import { LPNode } from '../lp';

const modules = {};

interface AstMap {
  [key: string]: LPNode;
}

class Module {
  moduleScope: Scope;
  exportScope: Scope;

  constructor(rootScope: Scope) {
    // Thoughts on how to handle this right now:
    // 1. The outermost module scope is read-only always.
    // 2. Therefore anything in the export scope can simply be duplicated in both scopes
    // 3. Therefore export scope needs access to the module scope so the functions function, but
    //    the module scope can just use its local copy
    this.moduleScope = new Scope(rootScope);
    this.exportScope = new Scope(this.moduleScope);
  }

  static getAllModules() {
    return modules;
  }

  static populateModule(
    path: string,
    ast: LPNode, // ModuleContext
    rootScope: Scope,
    isStd = false,
  ) {
    // First, take the export scope of the root scope and put references to it in this module. If
    // it is a built-in std module, it inherits from the root scope, otherwise it attaches all
    // exported references. This way std modules get access to the opcode scope via inheritance and
    // 'normal' modules do not.
    const module = new Module(isStd ? rootScope : undefined);
    if (!isStd) {
      for (const rootModuleName of Object.keys(rootScope.vals)) {
        module.moduleScope.put(rootModuleName, rootScope.vals[rootModuleName]);
      }
    }
    // Now, populate all of the imports
    const imports = ast.get('imports').getAll();
    for (const importAst of imports) {
      // If it's a "standard" import, figure out what name to call it (if the user overrode it)
      // and then attach the entire module with that name to the local scope.
      if (importAst.has('standardImport')) {
        const standardImport = importAst.get('standardImport');
        let importName: string;
        if (standardImport.get('renamed').has()) {
          importName = standardImport.get('renamed').get('varop').t;
        } else {
          const nameParts = standardImport.get('dependency').t.split('/');
          importName = nameParts[nameParts.length - 1];
        }
        const importedModule =
          modules[
            Ast.resolveDependency(
              path,
              importAst.get('standardImport').get('dependency'),
            )
          ];
        module.moduleScope.put(importName, importedModule.exportScope);
      }
      // If it's a "from" import, we're picking off pieces of the exported scope and inserting them
      // also potentially renaming them if requested by the user
      if (importAst.has('fromImport')) {
        const importedModule =
          modules[
            Ast.resolveDependency(
              path,
              importAst.get('fromImport').get('dependency'),
            )
          ];
        const vars = [];
        vars.push(
          importAst.get('fromImport').get('varlist').get('renameablevar'),
        );
        importAst
          .get('fromImport')
          .get('varlist')
          .get('cdr')
          .getAll()
          .forEach((r) => {
            vars.push(r.get('renameablevar'));
          });
        for (const moduleVar of vars) {
          const exportName = moduleVar.get('varop').t;
          let importName = exportName;
          if (moduleVar.get('renamed').has()) {
            importName = moduleVar.get('renamed').get('varop').t;
          }
          const thing = importedModule.exportScope.shallowGet(exportName);
          if (
            thing instanceof Array &&
            thing[0].microstatementInlining instanceof Function
          ) {
            const otherthing = module.moduleScope.deepGet(importName);
            if (
              !!otherthing &&
              otherthing instanceof Array &&
              (otherthing[0] as Fn).microstatementInlining instanceof Function
            ) {
              module.moduleScope.put(importName, [...thing, ...otherthing]);
            } else {
              module.moduleScope.put(importName, thing);
            }
          } else if (thing instanceof Array && thing[0] instanceof Operator) {
            const otherthing = module.moduleScope.deepGet(importName);
            if (
              !!otherthing &&
              otherthing instanceof Array &&
              otherthing instanceof Operator
            ) {
              module.moduleScope.put(importName, [...thing, ...otherthing]);
            } else {
              module.moduleScope.put(importName, thing);
            }
          } else {
            module.moduleScope.put(importName, thing);
          }
          // Special behavior for interfaces. If there are any functions or operators that match
          // the interface, pull them in. Similarly any types that match the entire interface. This
          // allows concise importing of a related suite of tools without having to explicitly call
          // out each one.
          if (thing instanceof Type && thing.iface) {
            const iface = thing.iface;
            const typesToCheck = Object.keys(importedModule.exportScope.vals)
              .map((n) => importedModule.exportScope.vals[n])
              .filter((v) => v instanceof Type);
            const fnsToCheck = Object.keys(importedModule.exportScope.vals)
              .map((n) => importedModule.exportScope.vals[n])
              .filter(
                (v) =>
                  v instanceof Array &&
                  v[0].microstatementInlining instanceof Function,
              );
            const opsToCheck = Object.keys(importedModule.exportScope.vals)
              .map((n) => importedModule.exportScope.vals[n])
              .filter((v) => v instanceof Array && v[0] instanceof Operator);

            typesToCheck
              .filter((t) => iface.typeApplies(t, importedModule.exportScope))
              .forEach((t) => {
                module.moduleScope.put(t.typename, t);
              });

            fnsToCheck
              .filter((fn) => {
                // TODO: Make this better and move it to the Interface file in the future
                return iface.functionTypes.some(
                  (ft: FunctionType) => ft.functionname === fn[0].getName(),
                );
              })
              .forEach((fn) => {
                module.moduleScope.put(fn[0].getName(), fn);
              });

            opsToCheck
              .filter((op) => {
                return iface.operatorTypes.some(
                  (ot: OperatorType) => ot.operatorname === op[0].name,
                );
              })
              .forEach((op) => {
                module.moduleScope.put(op[0].name, op);
              });
          }
        }
      }
    }
    const body = ast.get('body').getAll();
    // Next, types
    const types = body.filter((r) => r.has('types')).map((r) => r.get('types'));
    for (const typeAst of types) {
      const newType = Type.fromAst(typeAst, module.moduleScope);
      module.moduleScope.put(
        newType.typename,
        newType.alias ? newType.alias : newType,
      );
    }
    // Next, interfaces
    const interfaces = body
      .filter((r) => r.has('interfaces'))
      .map((r) => r.get('interfaces'));
    for (const interfaceAst of interfaces) {
      Interface.fromAst(interfaceAst, module.moduleScope);
      // Automatically inserts the interface into the module scope, we're done.
    }
    // Next, constants
    const constdeclarations = body
      .filter((r) => r.has('constdeclaration'))
      .map((r) => r.get('constdeclaration'));
    for (const constdeclaration of constdeclarations) {
      Constant.fromAst(constdeclaration, module.moduleScope);
    }
    // Next, events
    const events = body
      .filter((r) => r.has('events'))
      .map((r) => r.get('events'));
    for (const eventAst of events) {
      const newEvent = Event.fromAst(eventAst, module.moduleScope);
      module.moduleScope.put(newEvent.name, newEvent);
    }
    // Next, functions
    const functions = body
      .filter((r) => r.has('functions'))
      .map((r) => r.get('functions'));
    for (const functionAst of functions) {
      const newFunc = UserFunction.fromAst(functionAst, module.moduleScope);
      if (newFunc.getName() == null) {
        throw new Error('Module-level functions must have a name');
      }
      const fns = module.moduleScope.get(newFunc.getName()) as Array<Fn>;
      if (fns == null) {
        module.moduleScope.put(newFunc.getName(), [newFunc]);
      } else {
        fns.push(newFunc);
      }
    }
    // Next, operators
    const operatorMapping = body
      .filter((r) => r.has('operatormapping'))
      .map((r) => r.get('operatormapping'));
    for (const operatorAst of operatorMapping) {
      const isPrefix = operatorAst.get('fix').has('prefix');
      const name = operatorAst
        .get('opmap')
        .get()
        .get('fntoop')
        .get('operators')
        .t.trim();
      const precedence = parseInt(
        operatorAst.get('opmap').get().get('opprecedence').get('num').t,
        10,
      );
      const fns = module.moduleScope.deepGet(
        operatorAst.get('opmap').get().get('fntoop').get('fnname').t,
      ) as Array<Fn>;
      if (!fns) {
        throw new Error(
          'Operator ' +
            name +
            ' declared for unknown function ' +
            operatorAst.t,
        );
      }
      const op = new Operator(name, precedence, isPrefix, fns);
      const opsBox = module.moduleScope.deepGet(name) as Array<Operator>;
      if (!opsBox) {
        module.moduleScope.put(name, [op]);
      } else {
        // To make sure we don't accidentally mutate other scopes, we're cloning this operator list
        const ops = [...opsBox];
        ops.push(op);
        module.moduleScope.put(name, ops);
      }
    }
    // Next, exports, which can be most of the above
    const exports = body
      .filter((r) => r.has('exportsn'))
      .map((r) => r.get('exportsn').get('exportable'));
    for (const exportAst of exports) {
      if (exportAst.has('ref')) {
        const exportVar = module.moduleScope.deepGet(exportAst.get('ref').t);
        const splitName = exportAst.get('ref').t.split('.');
        module.moduleScope.put(splitName[splitName.length - 1], exportVar);
        module.exportScope.put(splitName[splitName.length - 1], exportVar);
      } else if (exportAst.has('types')) {
        const newType = Type.fromAst(
          exportAst.get('types'),
          module.moduleScope,
        );
        const typeBox = !newType.alias ? newType : newType.alias;
        module.moduleScope.put(newType.typename, typeBox);
        module.exportScope.put(newType.typename, typeBox);
      } else if (exportAst.has('interfaces')) {
        // Automatically inserts the interface into the module scope
        const interfaceBox = Interface.fromAst(
          exportAst.get('interfaces'),
          module.moduleScope,
        );
        module.exportScope.put(interfaceBox.typename, interfaceBox);
      } else if (exportAst.has('constdeclaration')) {
        const constVal = Constant.fromAst(
          exportAst.get('constdeclaration'),
          module.moduleScope,
        );
        module.exportScope.put(constVal.name, constVal);
      } else if (exportAst.has('functions')) {
        const newFunc = UserFunction.fromAst(
          exportAst.get('functions'),
          module.moduleScope,
        );
        if (!newFunc.getName()) {
          throw new Error(`Module-level functions must have a name:
${exportAst.get('functions').t}
`);
        }
        // Exported scope must be checked first because it will fall through to the not-exported
        // scope by default.
        const expFns = module.exportScope.shallowGet(
          newFunc.getName(),
        ) as Array<Fn>;
        if (!expFns) {
          module.exportScope.put(newFunc.getName(), [newFunc]);
        } else {
          expFns.push(newFunc);
        }
        const modFns = module.moduleScope.get(newFunc.getName()) as Array<Fn>;
        if (!modFns) {
          module.moduleScope.put(newFunc.getName(), [newFunc]);
        } else {
          modFns.push(newFunc);
        }
      } else if (exportAst.has('operatormapping')) {
        const operatorAst = exportAst.get('operatormapping');
        const isPrefix = operatorAst.get('fix').has('prefix');
        const name = operatorAst
          .get('opmap')
          .get()
          .get('fntoop')
          .get('operators')
          .t.trim();
        const precedence = parseInt(
          operatorAst.get('opmap').get().get('opprecedence').get('num').t,
          10,
        );
        let fns = module.moduleScope.deepGet(
          operatorAst.get('opmap').get().get('fntoop').get('fnname').t,
        ) as Array<Fn>;
        if (!fns) {
          fns = module.moduleScope.deepGet(
            operatorAst.get('opmap').get().get('fntoop').get('fnname').t,
          ) as Array<Fn>;
          if (fns) {
            throw new Error(
              'Exported operator ' +
                name +
                ' wrapping unexported function ' +
                operatorAst.get('opmap').get('fntoop').get('fnname').t +
                ' which is not allowed, please export the function, as well.',
            );
          }
          throw new Error(
            'Operator ' +
              name +
              ' declared for unknown function ' +
              operatorAst.get('opmap').get('fntoop').get('fnname').t,
          );
        }
        const op = new Operator(name, precedence, isPrefix, fns);
        const modOpsBox = module.moduleScope.deepGet(name) as Array<Operator>;
        if (!modOpsBox) {
          module.moduleScope.put(name, [op]);
        } else {
          const ops = [...modOpsBox];
          ops.push(op);
          module.moduleScope.put(name, ops);
        }
        const expOpsBox = module.exportScope.deepGet(name) as Array<Operator>;
        if (!expOpsBox) {
          module.exportScope.put(name, [op]);
        } else {
          const ops = [...expOpsBox];
          ops.push(op);
          module.exportScope.put(name, ops);
        }
      } else if (exportAst.has('events')) {
        const newEvent = Event.fromAst(
          exportAst.get('events'),
          module.moduleScope,
        );
        module.moduleScope.put(newEvent.name, newEvent);
        module.exportScope.put(newEvent.name, newEvent);
      } else {
        // What?
        throw new Error(
          'What should be an impossible export state has been reached.',
        );
      }
    }
    // Finally, event handlers, so they can depend on events that are exported from the same module
    const handlers = body
      .filter((r) => r.has('handlers'))
      .map((r) => r.get('handlers'));
    for (const handlerAst of handlers) {
      const evt = module.moduleScope.deepGet(handlerAst.get('eventname').t);
      if (!evt)
        throw new Error(
          'Could not find specified event: ' + handlerAst.get('eventname').t,
        );
      if (!(evt instanceof Event))
        throw new Error(handlerAst.get('eventname').t + ' is not an event');
      const handler = handlerAst.get('handler');
      let fn = null;
      if (handler.has('fnname')) {
        const fnName = handler.get('fnname').t;
        const fns = module.moduleScope.deepGet(fnName) as Array<Fn>;
        if (!fns)
          throw new Error('Could not find specified function: ' + fnName);
        if (
          !(
            fns instanceof Array &&
            fns[0].microstatementInlining instanceof Function
          )
        ) {
          throw new Error(fnName + ' is not a function');
        }
        for (let i = 0; i < fns.length; i++) {
          if (
            evt.type.typename === 'void' &&
            Object.values(fns[i].getArguments()).length === 0
          ) {
            fn = fns[i];
            break;
          }
          const argTypes = Object.values(fns[i].getArguments());
          if (argTypes.length !== 1) continue;
          if (argTypes[0] == evt.type) {
            fn = fns[i];
            break;
          }
        }
        if (fn == null) {
          throw new Error(
            'Could not find function named ' +
              fnName +
              ' with matching function signature',
          );
        }
      }
      if (handler.has('functions')) {
        fn = UserFunction.fromAst(handler.get('functions'), module.moduleScope);
      }
      if (handler.has('functionbody')) {
        fn = UserFunction.fromAst(
          handler.get('functionbody'),
          module.moduleScope,
        );
      }
      if (!fn) {
        // Shouldn't be possible
        throw new Error('Impossible state reached processing event handler');
      }
      if (
        Object.keys(fn.getArguments()).length > 1 ||
        (evt.type === Type.builtinTypes['void'] &&
          Object.keys(fn.getArguments()).length !== 0)
      ) {
        throw new Error(
          'Function provided for ' +
            handlerAst.get('eventname').t +
            ' has invalid argument signature',
        );
      }
      evt.handlers.push(fn);
    }
    return module;
  }

  static modulesFromAsts(astMap: AstMap, rootScope: Scope) {
    const modulePaths = Object.keys(astMap);
    while (modulePaths.length > 0) {
      for (let i = 0; i < modulePaths.length; i++) {
        const path = modulePaths[i];
        const moduleAst = astMap[path];
        const imports = Ast.resolveImports(path, moduleAst);
        let loadable = true;
        for (const importPath of imports) {
          if (importPath[0] === '@') continue;
          if (modules.hasOwnProperty(importPath)) continue;
          loadable = false;
        }
        if (!loadable) continue;
        modulePaths.splice(i, 1);
        i--;
        const module = Module.populateModule(path, moduleAst, rootScope);
        modules[path] = module;
      }
    }
    return modules;
  }
}

export default Module;
