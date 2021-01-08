import * as fs from 'fs'
import * as path from 'path'

import { LP, LPNode, LPError, } from '../lp'
import * as ln from '../ln'

const resolve = (path: string) => {
  try {
    return fs.realpathSync(path)
  } catch (e) {
    return null
  }
}

export const fromString = (str: string) => {
  const lp = LP.fromText(str)
  const ast = ln.ln.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  } else if (ast.t.length !== str.length) {
    const lp2 = lp.clone()
    lp2.advance(ast.t.length)
    const body = ast.get('body').getAll()
    const last = body[body.length - 1]
    throw new Error(`AST Parse error, cannot continue due to syntax error between line ${last.line}:${last.char} - ${lp2.line}:${lp2.char}`)
  }

  return ast
}

export const fromFile = (filename: string) => {
  console.error(`Parsing ${filename}`)
  const ast = fromString(fs.readFileSync(filename, { encoding: 'utf8', }))
  ast.filename = filename
  return ast
}

export const resolveDependency = (modulePath: string, dependency: LPNode) => {
  // Special case path for the standard library importing itself
  if (modulePath.substring(0, 4) === '@std') return dependency.t.trim()
  // For everything else...
  let importPath = null
  // If the dependency is a local dependency, there's little logic in determining
  // what is being imported. It's either the relative path to a file with the language
  // extension, or the relative path to a directory containing an "index.ln" file
  if (dependency.has('localdependency')) {
    const dirPath = resolve(path.join(
      path.dirname(modulePath),
      dependency.get('localdependency').t,
      "index.ln",
    ))
    const filePath = resolve(path.join(
      path.dirname(modulePath),
      dependency.get('localdependency').t + ".ln"
    ))
    // It's possible for both to exist. Prefer the directory-based one, but warn the user
    if (typeof dirPath === "string" && typeof filePath === "string") {
      console.error(dirPath + " and " + filePath + " both exist. Using " + dirPath)
    }
    if (typeof filePath === "string") {
      importPath = filePath
    }
    if (typeof dirPath === "string") {
      importPath = dirPath
    }
    if (importPath === null) {
      throw new Error(`The dependency ${dependency.get('localdependency').t} could not be found.`)
    }
  }
  // If the dependency is a global dependency, there's a more complicated resolution to find it.
  // This is inspired by the Ruby and Node resolution mechanisms, but with some changes that
  // should hopefully make some improvements so dependency-injection is effectively first-class
  // and micro-libraries are discouraged (the latter will require a multi-pronged effort)
  //
  // Essentially, there are two recursively-found directories that global modules can be found,
  // the `modules` directory and the `dependencies` directory (TBD: are these the final names?)
  // The `modules` directory is recursively checked first (with a special check to make sure it
  // ignores self-resolutions) and the first one found in that check, if any, is used. If not,
  // there's a special check if the dependency is an `@std/...` dependency, and if so to return
  // that string as-is so the built-in dependency is used. Next the same recursive check is
  // performed on the `dependencies` directories until the dependency is found. If that also
  // fails, then there will be a complaint and the process will exit.
  //
  // The idea is that the package manager will install dependencies into the `dependencies`
  // directory at the root of the project (or maybe PWD, but that seems a bit too unwieldy).
  // Meanwhile the `modules` directory will only exist if the developer wants it, but it can be
  // useful for cross-cutting code in the same project that doesn't really need to be open-
  // sourced but is annoying to always reference slightly differently in each file, eg
  // `../../../util`. Instead the project can have a project-root-level `modules` directory and
  // then `modules/util.ln` can be referenced simply with `import @util` anywhere in the project.
  //
  // Since this is also recursive, it's should make dependency injection a first-class citizen
  // of the language. For instance you can put all of your models in `modules/models/`, and then
  // your unit test suite can have its model mocks in `tests/modules/models/` and the dependency
  // you intend to inject into can be symlinked in the `tests/` directory to cause that version
  // to pull the injected code, instead. And of course, if different tests need different
  // dependency injections, you can turn the test file into a directory of the same name and
  // rename the file to `index.ln` within it, and then have the specific mocks that test needs
  // stored in a `modules/` directory in parallel with it, which will not impact other mocks.
  //
  // Because these mocks also have a special exception to not import themselves, this can also
  // be used for instrumentation purposes, where they override the actual module but then also
  // import the real thing and add extra behavior to it.
  //
  // While there are certainly uses for splitting some logical piece of code into a tree of
  // files and directories, it is my hope that the standard application organization path is a
  // project with a root `index.ln` file and `modules` and `dependencies` directories, and little
  // else. At least things like `modules/logger`, `modules/config`, etc should belong there.
  if (dependency.has('globaldependency')) {
    // Get the two potential dependency types, file and directory-style.
    const fileModule = dependency.get('globaldependency').t.substring(1) + ".ln"
    const dirModule = dependency.get('globaldependency').t.substring(1) + "/index.ln"
    // Get the initial root to check
    let pathRoot = path.dirname(modulePath)
    // Search the recursively up the directory structure in the `modules` directories for the
    // specified dependency, and if found, return it.
    while (pathRoot != null) {
      const dirPath = resolve(path.join(pathRoot, "modules", dirModule))
      const filePath = resolve(path.join(pathRoot, "modules", fileModule))
      // It's possible for a module to accidentally resolve to itself when the module wraps the
      // actual dependency it is named for.
      if (dirPath === modulePath || filePath === modulePath) {
        pathRoot = path.dirname(pathRoot)
        continue
      }
      // It's possible for both to exist. Prefer the directory-based one, but warn the user
      if (typeof dirPath === "string" && typeof filePath === "string") {
        console.error(dirPath + " and " + filePath + " both exist. Using " + dirPath)
      }
      if (typeof filePath === "string") {
        importPath = filePath
        break
      }
      if (typeof dirPath === "string") {
        importPath = dirPath
        break
      }
      if (pathRoot === "/" || /[A-Z]:\\/.test(pathRoot)) {
        pathRoot = null
      } else {
        pathRoot = path.dirname(pathRoot)
      }
    }
    if (importPath == null) {
      // If we can't find it defined in a `modules` directory, check if it's an `@std/...`
      // module and abort here so the built-in standard library is used.
      if (dependency.get('globaldependency').t.substring(0, 5) === "@std/") {
        // Not a valid path (starting with '@') to be used as signal to use built-in library)
        importPath = dependency.get('globaldependency').t
      } else {
        // Go back to the original point and search up the tree for `dependencies` directories
        pathRoot = path.dirname(modulePath)
        while (pathRoot != null) {
          const dirPath = resolve(path.join(pathRoot, "dependencies", dirModule))
          const filePath = resolve(path.join(pathRoot, "dependencies", fileModule))
          // It's possible for both to exist. Prefer the directory-based one, but warn the user
          if (typeof dirPath === "string" && typeof filePath === "string") {
            console.error(dirPath + " and " + filePath + " both exist. Using " + dirPath)
          }
          if (typeof filePath === "string") {
            importPath = filePath
            break
          }
          if (typeof dirPath === "string") {
            importPath = dirPath
            break
          }
          if (pathRoot === "/" || /[A-Z]:\\/.test(pathRoot)) {
            pathRoot = null
          } else {
            pathRoot = path.dirname(pathRoot)
          }
        }
      }
      if (importPath == null) {
        throw new Error(`The dependency ${dependency.get('globaldependency').t} could not be found.`)
      }
    }
  }
  return importPath
}

export const resolveImports = (modulePath: string, ast: LPNode) => {
  let resolvedImports = []
  let imports = ast.get('imports').getAll()
  for (let i = 0; i < imports.length; i++) {
    let dependency = null

    if (imports[i].has('standardImport')) {
      dependency = imports[i].get('standardImport').get('dependency')
    }
    if (imports[i].has('fromImport')) {
      dependency = imports[i].get('fromImport').get('dependency')
    }
    if (!dependency) {
      // Should I do anything else here?
      throw new Error('Malformed AST, import statement without an import definition?')
    }
    const importPath = resolveDependency(modulePath, dependency)
    resolvedImports.push(importPath)
  }
  return resolvedImports
}

export const functionAstFromString = (fn: string) => {
  const lp = LP.fromText(fn)
  const ast = ln.functions.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  } else if (ast.t.length !== fn.length) {
    const lp2 = lp.clone()
    lp2.advance(ast.t.length)
    throw new Error(`AST Parse error, cannot continue due to syntax error ending at line ${lp2.line}:${lp2.char}`)
  }

  return ast
}

export const statementAstFromString = (s: string) => {
  const lp = LP.fromText(s)
  const ast = ln.statement.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  } else if (ast.t.length !== s.length) {
    const lp2 = lp.clone()
    lp2.advance(ast.t.length)
    throw new Error(`AST Parse error, cannot continue due to syntax error ending at line ${lp2.line}:${lp2.char}`)
  }

  return ast
}

export const fulltypenameAstFromString = (s: string) => {
  const lp = LP.fromText(s)
  const ast = ln.fulltypename.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  } else if (ast.t.length !== s.length) {
    const lp2 = lp.clone()
    lp2.advance(ast.t.length)
    throw new Error(`AST Parse error, cannot continue due to syntax error ending at line ${lp2.line}:${lp2.char}`)
  }

  return ast
}

export const assignablesAstFromString = (s: string) => {
  const lp = LP.fromText(s)
  const ast = ln.assignables.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  } else if (ast.t.length !== s.length) {
    const lp2 = lp.clone()
    lp2.advance(ast.t.length)
    throw new Error(`AST Parse error, cannot continue due to syntax error ending at line ${lp2.line}:${lp2.char}`)
  }

  return ast
}

