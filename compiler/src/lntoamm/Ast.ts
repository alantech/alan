import * as fs from 'fs'
import * as path from 'path'

import { InputStream, CommonTokenStream, } from 'antlr4'

import { LP, } from '../lp'
import { LnLexer, LnParser, lp, stripcomments, } from '../ln'

const resolve = (path: string) => {
  try {
    return fs.realpathSync(path)
  } catch (e) {
    return null
  }
}

export const fromString = (str: string) => {
  const startTime = Date.now()
  // Perform debug parsing using the new parser and log errors (or success)
  const lpObj = LP.fromText(stripcomments(str))
  const ast = lp.apply(lpObj)
  const lpTime = Date.now()
  if (ast instanceof Error) {
    console.error(ast)
    console.error('str')
    console.error(str)
    console.error('stripped')
    console.error(stripcomments(str))
    console.error()
  } else {
    console.log(`LP-based LN parser success! Total time: ${lpTime - startTime}`)
    console.log(ast)
  }
  const inputStream = new InputStream(str)
  const langLexer = new LnLexer(inputStream)
  const commonTokenStream = new CommonTokenStream(langLexer)
  const langParser = new LnParser(commonTokenStream)
  const antlr = langParser.module()
  const antlrTime = Date.now()
  console.log(`ANTLR-based LN parse success! Total time: ${antlrTime - lpTime}`)

  return antlr
}

export const fromFile = (filename: string) => {
  return fromString(fs.readFileSync(filename, { encoding: 'utf8', }))
}

export const resolveDependency = (modulePath: string, dependency: any) => { // TODO: No ANTLR
  // Special case path for the standard library importing itself
  if (modulePath.substring(0, 4) === '@std') return dependency.getText().trim()
  // For everything else...
  let importPath = null
  // If the dependency is a local dependency, there's little logic in determining
  // what is being imported. It's either the relative path to a file with the language
  // extension, or the relative path to a directory containing an "index.ln" file
  if (dependency.localdependency() != null) {
    const dirPath = resolve(path.join(
      path.dirname(modulePath),
      dependency.localdependency().getText().toString(),
      "index.ln",
    ))
    const filePath = resolve(path.join(
      path.dirname(modulePath),
      dependency.localdependency().getText().toString() + ".ln"
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
      // Should I do anything else here?
      throw new Error(
        "The dependency " +
        dependency.localdependency().getText().toString() +
        " could not be found.")
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
  if (dependency.globaldependency() != null) {
    // Get the two potential dependency types, file and directory-style.
    const fileModule = dependency.globaldependency().getText().toString().substring(1) + ".ln"
    const dirModule = dependency.globaldependency().getText().toString().substring(1) + "/index.ln"
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
      if (dependency.globaldependency().getText().toString().substring(0, 5) === "@std/") {
        // Not a valid path (starting with '@') to be used as signal to use built-in library)
        importPath = dependency.globaldependency().getText().toString()
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
        // Should I do anything else here?
        throw new Error(
          "The dependency " +
          dependency.globaldependency().getText().toString() +
          " could not be found.")
      }
    }
  }
  return importPath
}

export const resolveImports = (modulePath: string, ast: any) => { // TODO: No ANTLR
  let resolvedImports = []
  let imports = ast.imports();
  for (let i = 0; i < imports.length; i++) {
    const standardImport = imports[i].standardImport()
    const fromImport = imports[i].fromImport()
    let dependency = null

    if (standardImport != null) {
      dependency = standardImport.dependency()
    }
    if (fromImport != null) {
      dependency = fromImport.dependency()
    }
    if (dependency == null) {
      // Should I do anything else here?
      throw new Error("Things are horribly broken!")
    }
    const importPath = resolveDependency(modulePath, dependency)
    resolvedImports.push(importPath)
  }
  return resolvedImports
}

export const functionAstFromString = (fn: string) => {
  const inputStream = new InputStream(fn)
  const langLexer = new LnLexer(inputStream);
  const commonTokenStream = new CommonTokenStream(langLexer)
  const langParser = new LnParser(commonTokenStream)

  return langParser.functions()
}

export const statementAstFromString = (s: string) => {
  const inputStream = new InputStream(s)
  const langLexer = new LnLexer(inputStream);
  const commonTokenStream = new CommonTokenStream(langLexer)
  const langParser = new LnParser(commonTokenStream)

  return langParser.statements()
}

export const fulltypenameAstFromString = (s: string) => {
  const inputStream = new InputStream(s)
  const langLexer = new LnLexer(inputStream);
  const commonTokenStream = new CommonTokenStream(langLexer)
  const langParser = new LnParser(commonTokenStream)

  return langParser.fulltypename()
}

export const assignablesAstFromString = (s: string) => {
  const inputStream = new InputStream(s)
  const langLexer = new LnLexer(inputStream);
  const commonTokenStream = new CommonTokenStream(langLexer)
  const langParser = new LnParser(commonTokenStream)

  return langParser.assignables()
}


