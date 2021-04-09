import * as fs from 'fs'

import * as Ast from './Ast'
import * as Std from './Std'
import { LPNode } from '../lp'
import Module from './Module';
import Event from './Event';

type ModuleAsts = {[path: string]: LPNode};

const ammFromModuleAsts = (asts: ModuleAsts): string => {
  let stdFiles: Set<string> = new Set();
  for (const [modPath, mod] of Object.entries(asts)) {
    for (const importt of Ast.resolveImports(modPath, mod)) {
      if (importt.substring(0, 5) === '@std/') {
        stdFiles.add(importt.substring(5, importt.length) + '.lnn')
      }
    }
  }
  Std.loadStdModules(stdFiles);
  const rootScope = Module.getAllModules()['<root>'].exportScope;
  Module.modulesFromAsts(asts, rootScope);
  Event.allEvents.forEach(event => event.typeCheck());
  return ''
}

const moduleAstsFromFile = (filename: string): ModuleAsts => {
  let moduleAsts: ModuleAsts = {};
  let paths = [];
  const rootPath = fs.realpathSync(filename);
  paths.push(rootPath);

  while (paths.length > 0) {
    const modulePath = paths.shift();
    let module = null;
    try {
      module = Ast.fromFile(modulePath);
    } catch (e) {
      console.error(`Could not load ${modulePath}`);
      throw e;
    }
    moduleAsts[modulePath] = module;
    const imports = Ast.resolveImports(modulePath, module);
    for (let i = 0; i < imports.length; i++) {
      if (!moduleAsts[imports[i]] && !(imports[i].substring(0, 5) === '@std/')) {
        paths.push(imports[i]);
      }
    }
  }
  return moduleAsts;
}

export const fromFile = (filename: string): string | Buffer => ammFromModuleAsts(moduleAstsFromFile(filename));

export const fromString = (str: string): string | Buffer => {
  return null;
}
