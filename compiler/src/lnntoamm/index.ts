import * as fs from 'fs';
import { LPNode } from '../lp';

import * as Ast from './Ast';

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

export const fromFile: (string) => string | Buffer = (filename) => ammFromModuleAsts(moduleAstsFromFile(filename));

export const fromString = (str: string): string | Buffer => {
  return null;
}
