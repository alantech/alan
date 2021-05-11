import { LPNode } from '../lp';
import * as Ast from './Ast';
import Const from './Const';
import Event from './Event';
import Fn from './Fn';
import Operator from './Operator';
import Scope, { isFunctions } from './Scope';
import Type from './Types';

const modules: {[name: string]: Module} = {}

interface AstMap {
  [key: string]: LPNode
}

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
    ast: LPNode, // ModuleContext
    rootScope: Scope,
    isStd: boolean = false,
  ): Module {
    // First, take the export scope of the root scope and put references to it in this module. If
    // it is a built-in std module, it inherits from the root scope, otherwise it attaches all
    // exported references. This way std modules get access to the opcode scope via inheritance and
    // 'normal' modules do not.
    let module = new Module(isStd ? rootScope : undefined);
    if (!isStd) {
      for (const [name, val] of Object.entries(rootScope.vals)) {
         module.moduleScope.put(name, val);
      }
    }

    // imports
    ast.get('imports').getAll().forEach(importAst => {
      if (importAst.has('standardImport')) {
        importAst = importAst.get('standardImport');
        let importName: string;
        if (importAst.get('renamed').has()) {
          importName = importAst.get('renamed').get('varop').t.trim();
        } else {
          const nameParts = importAst.get('dependency').t.trim().split('/');
          importName = nameParts.pop()
        }
        const resolved = Ast.resolveDependency(path, importAst.get('dependency'));
        const importedModule = modules[resolved];
        module.moduleScope.put(importName, importedModule.exportScope);
      } else {
        importAst = importAst.get('fromImport');
        const resolvedDep = Ast.resolveDependency(path, importAst.get('dependency'));
        const importedModule = modules[resolvedDep];
        let vars: LPNode[] = [];
        vars.push(importAst.get('varlist').get('renameablevar'));
        importAst.get('varlist')
            .get('cdr')
            .getAll()
            .forEach(r => vars.push(r.get('renameablevar')));
        vars.forEach(moduleVar => {
          const exportName = moduleVar.get('varop').t.trim();
          let importName = exportName;
          if (moduleVar.get('renamed').has()) {
            importName = moduleVar.get('renamed').get('varop').t.trim();
          }
          const thing = importedModule.exportScope.shallowGet(exportName);
          if (thing === null) {
            throw new Error(`couldn't import ${exportName}: not defined in ${resolvedDep}`)
          } else if (isFunctions(thing)) {
            const otherthing = module.moduleScope.deepGet(importName);
            if (otherthing === null) {
              module.moduleScope.put(importName, [...thing]);
            } else if (isFunctions(otherthing)) {
              // note: this was `...thing, ...otherthing` before, but that
              // breaks preference for more-recently-defined things
              module.moduleScope.put(importName, [...otherthing, ...thing]);
            } else {
              throw new Error(`incompatible imports for ${importName}`);
            }
          } else {
            module.moduleScope.put(importName, thing);
          }
        });
      }
    });

    // now we're done with imports, move on to the body
    const body = ast.get('body').getAll();
    body.filter(r => r.has('types')).forEach(a => module.addTypeAst(a, false));
    body.filter(r => r.has('interfaces')).forEach(a => module.addInterfaceAst(a, false));
    body.filter(r => r.has('constdeclaration')).forEach(a => module.addConstAst(a, false));
    body.filter(r => r.has('events')).forEach(a => module.addEventAst(a, false));
    body.filter(r => r.has('functions')).forEach(a => module.addFnAst(a, false));
    body.filter(r => r.has('operatormapping')).forEach(a => module.addOpAst(a, false));
    body.filter(r => r.has('exportsn')).forEach(node => {
      node = node.get('exportsn').get('exportable');
      if (node.has('ref')) {
        const ref = node.get('ref');
        const exportVar = module.moduleScope.deepGet(ref.t.trim());
        const name = ref.t.trim().split('.').pop();
        module.moduleScope.put(name, exportVar);
        module.exportScope.put(name, exportVar);
      } else if (node.has('types')) {
        module.addTypeAst(node, true);
      } else if (node.has('interfaces')) {
        module.addInterfaceAst(node, true);
      } else if (node.has('constdeclaration')) {
        module.addConstAst(node, true);
      } else if (node.has('events')) {
        module.addEventAst(node, true);
      } else if (node.has('functions')) {
        module.addFnAst(node, true);
      } else if (node.has('operatormapping')) {
        module.addOpAst(node, true);
      }
    });
    body.filter(r => r.has('handlers')).forEach(a => module.addHandlerAst(a));

    return module;
  }

  static modulesFromAsts(
    astMap: AstMap,
    rootScope: Scope,
  ) {
    let modulePaths = Object.keys(astMap);
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

  addTypeAst(typeAst: LPNode, isExport: boolean) {
    typeAst = typeAst.get('types');
    const newType = Type.fromTypesAst(typeAst, this.moduleScope);
    this.moduleScope.put(newType.name, newType);
    if (isExport) {
      this.exportScope.put(newType.name, newType);
    }
  }

  addInterfaceAst(interfaceAst: LPNode, isExport: boolean) {
    interfaceAst = interfaceAst.get('interfaces');
    const newInterface = Type.fromInterfacesAst(interfaceAst, this.moduleScope);
    this.moduleScope.put(newInterface.name, newInterface);
    if (isExport) {
      this.exportScope.put(newInterface.name, newInterface);
    }
  }

  addConstAst(constAst: LPNode, isExport: boolean) {
    constAst = constAst.get('constdeclaration');
    const newConst = Const.fromAst(constAst, this.moduleScope);
    this.moduleScope.put(newConst.name, newConst);
    if (isExport) {
      this.exportScope.put(newConst.name, newConst);
    }
  }

  addEventAst(eventAst: LPNode, isExport: boolean) {
    eventAst = eventAst.get('events');
    const newEvent = Event.fromAst(eventAst, this.moduleScope);
    this.moduleScope.put(newEvent.name, newEvent);
    if (isExport) {
      this.exportScope.put(newEvent.name, newEvent);
    }
  }

  addFnAst(fnAst: LPNode, isExport: boolean) {
    fnAst = fnAst.get('functions');
    const newFn = Fn.fromFunctionsAst(fnAst, this.moduleScope);
    if (newFn.name === null) {
      throw new Error('Module-level functions must have a name');
    }
    let insertScopes = [this.moduleScope];
    if (isExport) insertScopes.push(this.exportScope);
    for (let scope of insertScopes) {
      const otherFns = scope.get(newFn.name) || [];
      if (!(otherFns instanceof Array)) {
        throw new Error(`Tried to define function ${newFn.name}, but a non-function by that name is already in scope`);
      } else if (otherFns.length > 0 && !(otherFns[0] instanceof Fn)) {
        throw new Error(`Tried to define function ${newFn.name}, but a non-function by that name is already in scope`);
      }
      scope.put(newFn.name, [...otherFns, newFn]);
    }
  }

  addOpAst(opAst: LPNode, isExport: boolean) {
    opAst = opAst.get('operatormapping');
    const newOp = Operator.fromAst(opAst, this.moduleScope);
    // no need to validate this since only operators can have such a name
    const otherOps = this.moduleScope.get(newOp.symbol) || [];
    this.moduleScope.put(newOp.symbol, [...otherOps, newOp]);
    if (isExport) {
      const exportedOps = this.exportScope.get(newOp.symbol) || [];
      this.moduleScope.put(newOp.symbol, [...exportedOps, newOp]);
    }
  }

  addHandlerAst(handlerAst: LPNode) {
    handlerAst = handlerAst.get('handlers');
    const eventName = handlerAst.get('eventname').t;
    let event = this.moduleScope.deepGet(eventName);
    if (event === null) {
      throw new Error(`Could not find specified event: ${eventName}`);
    } else if (!(event instanceof Event)) {
      throw new Error(`${eventName} is not an event`);
    }

    handlerAst = handlerAst.get('handler');
    let fns: Fn | Fn[] = null;
    if (handlerAst.has('fnname')) {
      const fnName = handlerAst.get('fnname').t;
      const inScope = this.moduleScope.deepGet(fnName);
      if (inScope === null) {
        throw new Error(`Could not find specified function: ${fnName}`);
      } else if (!(inScope instanceof Array) || !(inScope[0] instanceof Fn)) {
        throw new Error(`${fnName} is not a function`);
      }
      fns = inScope as Fn[];
    } else if (handlerAst.has('functions')) {
      fns = Fn.fromFunctionsAst(handlerAst.get('functions'), this.moduleScope);
    } else if (handlerAst.has('functionbody')) {
      fns = Fn.fromFunctionbody(handlerAst.get('functionbody'), this.moduleScope);
    }
    // gets type-checked later
    event.handlers.push(fns);
  }
}

export default Module
