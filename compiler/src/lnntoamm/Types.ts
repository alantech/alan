import { LPNode } from "../lp";
import Scope from "./Scope";
import { TODO } from "./util";

type TypeOrIface = Type | Interface;
type Fields = {[name: string]: Type};
type FunctionType = { args: TypeOrIface[], ret: Type };
type GenericArgs = {[name: string]: Type | null};
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

export class Type {
  name: string
  alias: Type | null
  genericArgs: GenericArgs
  fields: Fields

  constructor(
    name: string,
    genericArgs: GenericArgs,
    alias: Type | null,
    fields: Fields,
  ) {
    this.name = name;
    this.genericArgs = genericArgs;
    this.alias = alias;
    this.fields = fields;
  }

  static fromAst(ast: LPNode, scope: Scope): Type {
    const names = parseFulltypename(ast.get('fulltypename'));
    if (names[1].some(ty => ty[1].length !== 0)) {
      throw new Error(`Generic type variables can't have generic type arguments`);
    }
    const typeName = names[0];
    let genericArgs: GenericArgs = {};
    names[1].forEach(n => genericArgs[n[0]] = null);

    ast = ast.get('typedef');
    if (ast.has('typebody')) {
      ast = ast.get('typebody').get('typelist');
      const lines = [
        ast.get('typeline'),
        ...ast.get('cdr').getAll().map(n => n.get('typeline'))
      ];
      let fields: Fields = {};
      lines.forEach(line => {
        const fieldName = line.get('variable').t.trim();
        const fieldTy = this.getFromTypename(line.get('fulltypename'), scope);
        if (fieldTy instanceof Interface) {
          throw new Error(`type fields can't be interfaces (I think)`);
        }
        fields[fieldName] = fieldTy;
      });
      return new Type(typeName, genericArgs, null, fields);
    } else {
      ast = ast.get('typealias');
      TODO('type aliases in type construction');
    }
  }

  static getFromTypename(ast: LPNode, scope: Scope): Interface | Type {
    return null;
  }
}

type InterfaceOps = {};
type InterfaceFns = {[name: string]: FunctionType};

export class Interface {
  name: string
  alias: Interface | null
  properties: Fields
  functions: InterfaceFns
  operators: InterfaceOps

  constructor(
    name: string,
    alias: Interface | null = null,
    properties: Fields,
    functions: InterfaceFns,
    operators: InterfaceOps,
  ) {
    this.name = name;
    this.alias = alias;
    this.properties = properties;
    this.functions = functions;
    this.operators = operators;
  }

  static fromAst(ast: LPNode, scope: Scope): Interface {
    const name = ast.get('variable').t.trim();
    ast = ast.get('interfacedef');
    if (ast.has('interfacebody')) {
      ast = ast.get('interfacebody').get('interfacelist');
      const lines = [
        ast.get('interfaceline'),
        ...ast.get('cdr').getAll(),
      ];
      let properties: Fields = {};
      let functions: InterfaceFns = {};
      let operators: InterfaceOps = {};
      lines.forEach(lineAst => {
        if (lineAst.has('propertytypeline')) {
          lineAst = lineAst.get('propertytypeline');
          const propName = lineAst.get('variable').t.trim();
          const propTy = Type.getFromTypename(lineAst.get('fulltypename'), scope);
          if (propTy instanceof Interface) {
            TODO('interfaces can probably have interface fields');
            // necessary because ts doesn't realize that TODO means everything past
            // the call is unreachable :(
            return;
          }
          properties[propName] = propTy;
        } else if (lineAst.has('functiontypeline')) {
          lineAst = lineAst.get('functiontypeline');
          const fnName = lineAst.get('variable').t.trim();
          lineAst = lineAst.get('functiontype');
          const argTys = lineAst.get('cdr').getAll().map(argTyAst => Type.getFromTypename(argTyAst.get('fulltypename'), scope));
          const retTy = Type.getFromTypename(lineAst.get('returntype'), scope);
          if (retTy instanceof Interface) {
            throw new Error(`functions can't return interfaces`);
          }
          functions[fnName] = {args: argTys, ret: retTy};
        } else {
          lineAst = lineAst.get('operatortypeline');
          TODO('interface operators');
        }
      });
      return new Interface(
        name,
        null,
        properties,
        functions,
        operators,
      );
    } else {
      ast = ast.get('interfacealias');
      TODO('interface alias');
    }
  }
}
