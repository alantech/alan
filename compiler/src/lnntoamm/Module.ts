import { LPNode } from '../lp';
import * as Ast from './Ast';
import Const from './Const';
import Event from './Event';
import Fn from './Fn';
import Operator from './Operator';
import Scope from './Scope';
import { Interface, Type } from './Types';

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
      } else if (importAst.has('fromImport')) {
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
          throw new Error(`can't do from imports yet`)
        });
      }
    });

    // now we're done with imports, move on to the body
    const body = ast.get('body').getAll();

    // type
    const onTypeAst = (insertScope: Scope) => (typeAst: LPNode) => {
      typeAst = typeAst.get('types');
      const newType = Type.fromAst(typeAst, module.moduleScope);
      insertScope.put(newType.name, newType);
    };
    body.filter(r => r.has('types')).forEach(onTypeAst(module.moduleScope));

    // interface
    const onInterfaceAst = (insertScope: Scope) => (interfaceAst: LPNode) => {
      interfaceAst = interfaceAst.get('interfaces');
      const newInterface = Interface.fromAst(interfaceAst, module.moduleScope);
      insertScope.put(newInterface.name, newInterface);
    };
    body.filter(r => r.has('interfaces')).forEach(onInterfaceAst(module.moduleScope));

    // const
    const onConstAst = (insertScope: Scope) => (constAst: LPNode) => {
      constAst = constAst.get('constdeclaration');
      const newConst = Const.fromAst(constAst, module.moduleScope);
      insertScope.put(newConst.name, newConst);
    };
    body.filter(r => r.has('constdeclaration')).forEach(onConstAst(module.moduleScope))

    // event
    const onEventAst = (insertScope: Scope) => (eventAst: LPNode) => {
      eventAst = eventAst.get('events');
      const newEvent = Event.fromAst(eventAst, module.moduleScope);
      insertScope.put(newEvent.name, newEvent);
    };
    body.filter(r => r.has('events')).forEach(onEventAst(module.moduleScope));

    // fn
    const onFnAst = (insertScope: Scope) => (fnAst: LPNode) => {
      fnAst = fnAst.get('functions');
      const newFn = Fn.fromFunctionsAst(fnAst, module.moduleScope);
      if (newFn.name === null) {
        throw new Error('Module-level functions must have a name');
      }
      const otherFns = module.moduleScope.get(newFn.name) || new Array<Fn>();
      if (!(otherFns instanceof Array)) {
        throw new Error('Only functions can have the same name at the module level');
      }
      if (otherFns.length > 0 && !(otherFns[0] instanceof Fn)) {
        throw new Error('Only functions can have the same name at the module level');
      }
      insertScope.put(newFn.name, [...otherFns, newFn]);
    };
    body.filter(r => r.has('functions')).forEach(onFnAst(module.moduleScope));

    // operator
    const onOpAst = (insertScope: Scope) => (opAst: LPNode) => {
      opAst = opAst.get('operatormapping');
      const newOp = Operator.fromAst(opAst, module.moduleScope);
      const otherOps = module.moduleScope.get(newOp.name) || new Array<Operator>();
      insertScope.put(newOp.name, [...otherOps, newOp]);
    };
    body.filter(r => r.has('operatormapping')).forEach(onOpAst(module.moduleScope));

    // export
    body.filter(r => r.has('exportsn')).forEach(node => {
      node = node.get('exportable');
      if (node.has('ref')) {
        const ref = node.get('ref');
        const exportVar = module.moduleScope.deepGet(ref.t.trim());
        const name = ref.t.trim().split('.').pop();
        module.moduleScope.put(name, exportVar);
        module.exportScope.put(name, exportVar);
      } else if (node.has('types')) {
        onTypeAst(module.exportScope)(node);
      } else if (node.has('interfaces')) {
        onInterfaceAst(module.exportScope)(node);
      } else if (node.has('constdeclaration')) {
        onConstAst(module.exportScope)(node);
      } else if (node.has('events')) {
        onEventAst(module.exportScope)(node);
      } else if (node.has('functions')) {
        onFnAst(module.exportScope)(node);
      } else if (node.has('operatormapping')) {
        onOpAst(module.exportScope)(node);
      }
    });

    // on event handler
    body.filter(r => r.has('handlers')).forEach(handlerAst => {
      handlerAst = handlerAst.get('handlers');
      const eventName = handlerAst.get('eventname').t.trim();
      let event = module.moduleScope.deepGet(eventName);
      if (event === null) {
        throw new Error(`Could not find specified event: ${eventName}`);
      } else if (!(event instanceof Event)) {
        throw new Error(`${eventName} is not an event`);
      }

      handlerAst = handlerAst.get('handler');
      let fns: Fn | Fn[] = null;
      if (handlerAst.has('fnname')) {
        const fnName = handlerAst.get('fnname').t.trim();
        const asScoped = module.moduleScope.deepGet(fnName);
        if (asScoped === null) {
          throw new Error(`Could not find specified function: ${fnName}`);
        } else if (!(asScoped instanceof Array) || !(asScoped[0] instanceof Fn)) {
          throw new Error(`${fnName} is not a function`);
        }
        fns = asScoped as Fn[];
      } else if (handlerAst.has('functions')) {
        const fn = Fn.fromFunctionsAst(handlerAst.get('functions'), module.moduleScope);
        fns = fn;
      } else if (handlerAst.has('functionbody')) {
        const fn = Fn.fromFunctionbody(handlerAst.get('functionbody'), module.moduleScope);
        fns = fn;
      }
      // gets type-checked later
      event.handlers.push(fns);
    });

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
}

export default Module
