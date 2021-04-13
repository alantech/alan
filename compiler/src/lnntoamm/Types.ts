import { LPNode } from "../lp";
import Scope from "./Scope";
import { genName, TODO } from "./util";

type Fields = {[name: string]: Type | null};
export type GenericArgs = {[name: string]: Type | null};
type TypeName = [string, TypeName[]];

const parseFulltypename = (node: LPNode): TypeName => {
  const name = node.get('typename').t.trim();
  let genericTys: TypeName[] = [];
  if (node.get('opttypegenerics').has()) {
    const generics = node.get('opttypegenerics').get();
    genericTys.push(parseFulltypename(generics.get('fulltypename')));
    generics.get('cdr').getAll().map(n => n.get('fulltypename'));
  }
  return [name, genericTys];
};

export default abstract class Type {
  name: string
  alias: Type | null

  constructor(
    name: string,
    alias: Type | null,
  ) {
    this.name = name;
    this.alias = alias;
  }

  static fromInterfacesAst(ast: LPNode, scope: Scope): Type {
    return Interface.fromAst(ast, scope);
  }

  static fromTypesAst(ast: LPNode, scope: Scope): Type {
    return Struct.fromAst(ast, scope);
  }

  static generate(): Type {
    return new Generated();
  }

  static oneOf(types: Type[]): Type {
    return new OneOf(types);
  }

  static newBuiltin(name: string, generics: string[]): Type {
    let genericArgs: GenericArgs = {};
    generics.forEach(arg => genericArgs[arg] = null);
    return new Struct(
      name,
      null,
      genericArgs,
      {},
    );
  }

  static getFromTypename(name: LPNode | string, scope: Scope): Type {
    return scope.get(name.toString().trim());
  }

  abstract compatibleWithConstraint(that: Type): boolean;
}

export class FunctionType extends Type {
  argTys: Type[]
  retTy: Type
  callSelect: Type

  constructor(
    name: string,
    argTys: Type[],
    retTy: Type,
  ) {
    super(name, null);
    this.argTys = argTys;
    this.retTy = retTy;
  }

  compatibleWithConstraint(that: Type): boolean {
    return TODO()
  }
}

class Struct extends Type {
  ast: LPNode
  generics: GenericArgs
  properties: Fields

  constructor(
    name: string,
    ast: LPNode,
    genericArgs: GenericArgs,
    properties: Fields,
  ) {
    super(name, null);
    this.ast = ast;
    this.generics = genericArgs;
    this.properties = properties;
  }

  static fromAst(ast: LPNode, scope: Scope): Struct {
    let work = ast;
    const names = parseFulltypename(work.get('fulltypename'));
    if (names[1].some(ty => ty[1].length !== 0)) {
      throw new Error(`Generic type variables can't have generic type arguments`);
    }
    const typeName = names[0];
    let genericArgs: GenericArgs = {};
    names[1].forEach(n => genericArgs[n[0]] = null);

    work = ast.get('typedef');
    if (work.has('typebody')) {
      work = work.get('typebody').get('typelist');
      const lines = [
        work.get('typeline'),
        ...work.get('cdr').getAll().map(n => n.get('typeline')),
      ];
      let fields: Fields = {};
      lines.forEach(line => {
        const fieldName = line.get('variable').t;
        const fieldTy = Type.getFromTypename(line.get('fulltypename'), scope);
        if (fieldTy instanceof Interface) {
          throw new Error(`type fields can't be interfaces (I think)`);
        }
        fields[fieldName] = fieldTy;
      });
      return new Struct(typeName, ast, genericArgs, fields);
    } else {
      ast = ast.get('typealias');
      TODO('type aliases');
    }
  }

  compatibleWithConstraint(that: Type): boolean {
    if (that instanceof Struct) {
      return this === that;
    } else {
      TODO()
    }
  }
}

class Interface extends Type {
  constructor(
    name: string,
  ) {
    super(name, null);
  }

  static fromAst(ast: LPNode, scope: Scope): Interface {
    return null;
  }

  compatibleWithConstraint(that: Type): boolean {
    return TODO()
  }
}

// Types not available in Alan, but we must use anyways
class Generated extends Type {
  delegate: Type | null

  constructor() {
    super(genName(), null);
    this.delegate = null;
  }

  compatibleWithConstraint(that: Type): boolean {
    if (this.delegate === null) {
      this.delegate = that;
      return true;
    } else {
      return this.delegate.compatibleWithConstraint(that);
    }
  }
}

class OneOf extends Type {
  possibilities: Type[]

  constructor(
    possibilities: Type[],
  ) {
    super(genName(), null);
    this.possibilities = possibilities;
  }

  compatibleWithConstraint(that: Type): boolean {
    this.possibilities = this.possibilities.filter(ty => ty.compatibleWithConstraint(that));
    return this.possibilities.length > 0;
  }
}

// type TypeOrIface = Type | Interface;
// type Fields = {[name: string]: Type};
// type FunctionType = { args: TypeOrIface[], ret: Type };
// type GenericArgs = {[name: string]: Type | null};
// type TypeName = [string, TypeName[]];

// class Type {
//   name: string
//   alias: Type | null
//   genericArgs: GenericArgs
//   fields: Fields

//   constructor(
//     name: string,
//     genericArgs: GenericArgs,
//     alias: Type | null,
//     fields: Fields,
//   ) {
//     this.name = name;
//     this.genericArgs = genericArgs;
//     this.alias = alias;
//     this.fields = fields;
//   }

//   static fromAst(ast: LPNode, scope: Scope): Type {
//     const names = parseFulltypename(ast.get('fulltypename'));
//     if (names[1].some(ty => ty[1].length !== 0)) {
//       throw new Error(`Generic type variables can't have generic type arguments`);
//     }
//     const typeName = names[0];
//     let genericArgs: GenericArgs = {};
//     names[1].forEach(n => genericArgs[n[0]] = null);

//     ast = ast.get('typedef');
//     if (ast.has('typebody')) {
//       ast = ast.get('typebody').get('typelist');
//       const lines = [
//         ast.get('typeline'),
//         ...ast.get('cdr').getAll().map(n => n.get('typeline'))
//       ];
//       let fields: Fields = {};
//       lines.forEach(line => {
//         const fieldName = line.get('variable').t.trim();
//         const fieldTy = this.getFromTypename(line.get('fulltypename'), scope);
//         if (fieldTy instanceof Interface) {
//           throw new Error(`type fields can't be interfaces (I think)`);
//         }
//         fields[fieldName] = fieldTy;
//       });
//       return new Type(typeName, genericArgs, null, fields);
//     } else {
//       ast = ast.get('typealias');
//       TODO('type aliases in type construction');
//     }
//   }

//   static getFromTypename(ast: LPNode, scope: Scope): Interface | Type {
//     return null;
//   }
// }

// type InterfaceOps = {};
// type InterfaceFns = {[name: string]: FunctionType};

// class Interface {
//   name: string
//   alias: Interface | null
//   properties: Fields
//   functions: InterfaceFns
//   operators: InterfaceOps

//   constructor(
//     name: string,
//     alias: Interface | null = null,
//     properties: Fields,
//     functions: InterfaceFns,
//     operators: InterfaceOps,
//   ) {
//     this.name = name;
//     this.alias = alias;
//     this.properties = properties;
//     this.functions = functions;
//     this.operators = operators;
//   }

//   static fromAst(ast: LPNode, scope: Scope): Interface {
//     const name = ast.get('variable').t.trim();
//     ast = ast.get('interfacedef');
//     if (ast.has('interfacebody')) {
//       ast = ast.get('interfacebody').get('interfacelist');
//       const lines = [
//         ast.get('interfaceline'),
//         ...ast.get('cdr').getAll(),
//       ];
//       let properties: Fields = {};
//       let functions: InterfaceFns = {};
//       let operators: InterfaceOps = {};
//       lines.forEach(lineAst => {
//         if (lineAst.has('propertytypeline')) {
//           lineAst = lineAst.get('propertytypeline');
//           const propName = lineAst.get('variable').t.trim();
//           const propTy = Type.getFromTypename(lineAst.get('fulltypename'), scope);
//           if (propTy instanceof Interface) {
//             TODO('interfaces can probably have interface fields');
//             // necessary because ts doesn't realize that TODO means everything past
//             // the call is unreachable :(
//             return;
//           }
//           properties[propName] = propTy;
//         } else if (lineAst.has('functiontypeline')) {
//           lineAst = lineAst.get('functiontypeline');
//           const fnName = lineAst.get('variable').t.trim();
//           lineAst = lineAst.get('functiontype');
//           const argTys = lineAst.get('cdr').getAll().map(argTyAst => Type.getFromTypename(argTyAst.get('fulltypename'), scope));
//           const retTy = Type.getFromTypename(lineAst.get('returntype'), scope);
//           if (retTy instanceof Interface) {
//             throw new Error(`functions can't return interfaces`);
//           }
//           functions[fnName] = {args: argTys, ret: retTy};
//         } else {
//           lineAst = lineAst.get('operatortypeline');
//           TODO('interface operators');
//         }
//       });
//       return new Interface(
//         name,
//         null,
//         properties,
//         functions,
//         operators,
//       );
//     } else {
//       ast = ast.get('interfacealias');
//       TODO('interface alias');
//     }
//   }
// }
