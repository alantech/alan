import Operator from './Operator';
import Scope from './Scope';
import { Fn } from './Function';
import { fulltypenameAstFromString } from './Ast';
import { LPNode } from '../lp';

type Properties = {
  [K: string]: Type;
};

type Generics = {
  [K: string]: number;
};

export class FunctionType {
  functionname: string | null;
  args: Array<Type>;
  returnType: Type;

  constructor(
    functionname: string | null = null,
    args: Array<Type> = [],
    returnType: Type,
  ) {
    this.functionname = functionname;
    this.args = args;
    this.returnType = returnType;
  }
}

export class OperatorType {
  operatorname: string | null;
  isPrefix: boolean;
  args: Array<Type>;
  returnType: Type;

  constructor(
    operatorname: string,
    isPrefix = false,
    args: Array<Type>,
    returnType: Type,
  ) {
    this.operatorname = operatorname;
    this.isPrefix = isPrefix;
    this.args = args;
    this.returnType = returnType;
  }
}

export class Interface {
  interfacename: string;
  functionTypes: Array<FunctionType>;
  operatorTypes: Array<OperatorType>;
  requiredProperties: Properties;

  constructor(
    interfacename: string,
    functionTypes: Array<FunctionType> = [],
    operatorTypes: Array<OperatorType> = [],
    requiredProperties: Properties = {},
  ) {
    this.interfacename = interfacename;
    this.functionTypes = functionTypes;
    this.operatorTypes = operatorTypes;
    this.requiredProperties = requiredProperties;
  }

  typeApplies(typeToCheck: Type, scope: Scope) {
    // Solve circular dependency issue
    for (const requiredProperty of Object.keys(this.requiredProperties)) {
      if (!typeToCheck.properties.hasOwnProperty(requiredProperty))
        return false;
    }

    for (const functionType of this.functionTypes) {
      if (!functionType.functionname) continue; // Anonymous functions checked at callsite
      const potentialFunctions = scope.deepGet(
        functionType.functionname,
      ) as Array<Fn>;
      if (
        !potentialFunctions ||
        !(
          potentialFunctions instanceof Array &&
          potentialFunctions[0].microstatementInlining instanceof Function
        )
      ) {
        throw new Error(
          functionType.functionname + ' is not the name of a function',
        );
      }
      let functionFound = false;
      for (const potentialFunction of potentialFunctions) {
        const argTypes = potentialFunction.getArguments();
        let argsMatch = true;
        const typeNames = Object.keys(argTypes);
        for (let i = 0; i < typeNames.length; i++) {
          const functionTypeArgType = functionType.args[i];
          if (argTypes[typeNames[i]] === functionTypeArgType) continue;
          if (argTypes[typeNames[i]].originalType === functionTypeArgType)
            continue;
          if (
            argTypes[typeNames[i]].originalType ===
              functionTypeArgType.originalType &&
            Object.values(functionTypeArgType.properties).every((prop, j) => {
              const comparable = Object.values(
                argTypes[typeNames[i]].properties,
              )[j];
              if (prop === comparable) return true;
              if (prop.iface && prop.iface.typeApplies(comparable, scope))
                return true;
              return false;
            })
          )
            continue;
          if (argTypes[typeNames[i]] === typeToCheck) continue;
          if (
            !!argTypes[typeNames[i]].iface &&
            !!functionTypeArgType.iface &&
            argTypes[typeNames[i]].iface === functionTypeArgType.iface
          )
            continue;
          argsMatch = false;
          break;
        }
        if (!argsMatch) continue;
        functionFound = true;
        break;
      }
      if (!functionFound) return false;
    }

    for (const operatorType of this.operatorTypes) {
      const potentialOperators = scope.deepGet(
        operatorType.operatorname,
      ) as Array<Operator>;
      if (
        !potentialOperators ||
        !(
          potentialOperators instanceof Array &&
          potentialOperators[0] instanceof Operator
        )
      ) {
        throw new Error(`${operatorType.operatorname} is not an operator`);
      }
      let operatorFound = false;
      for (const potentialOperator of potentialOperators) {
        for (const potentialFunction of potentialOperator.potentialFunctions) {
          const argTypes = potentialFunction.getArguments();
          let argsMatch = true;
          const typeNames = Object.keys(argTypes);
          for (let i = 0; i < typeNames.length; i++) {
            const operatorTypeArgType = operatorType.args[i];
            if (argTypes[typeNames[i]] === operatorTypeArgType) continue;
            if (argTypes[typeNames[i]].originalType === operatorTypeArgType)
              continue;
            if (argTypes[typeNames[i]] === typeToCheck) continue;
            if (
              !!argTypes[typeNames[i]].iface &&
              !!operatorTypeArgType.iface &&
              argTypes[typeNames[i]].iface === operatorTypeArgType.iface
            )
              continue;
            argsMatch = false;
            break;
          }
          if (!argsMatch) continue;
          operatorFound = true;
          break;
        }
      }
      if (!operatorFound) return false;
    }

    return true;
  }

  static fromAst(interfaceAst: LPNode, scope: Scope) {
    // Construct the basic interface, the wrapper type, and insert it into the scope
    // This is all necessary so the interface can self-reference when constructing the function and
    // operator types.
    const interfacename = interfaceAst.get('variable').t;
    const iface = new Interface(interfacename);
    const ifaceType = new Type(
      interfacename,
      false,
      false,
      {},
      {},
      null,
      iface,
    );
    scope.put(interfacename, ifaceType);

    // Now, insert the actual declarations of the interface, if there are any (if there are none,
    // it will provide only as much as a type generic -- you can set it to a variable and return it
    // but nothing else, unlike Go's ridiculous interpretation of a bare interface).
    if (
      interfaceAst.get('interfacedef').has('interfacebody') &&
      interfaceAst
        .get('interfacedef')
        .get('interfacebody')
        .get('interfacelist')
        .has()
    ) {
      const interfacelist = interfaceAst
        .get('interfacedef')
        .get('interfacebody')
        .get('interfacelist');
      const interfacelines = [];
      interfacelines.push(interfacelist.get('interfaceline'));
      interfacelist
        .get('cdr')
        .getAll()
        .forEach((l) => {
          interfacelines.push(l.get('interfaceline'));
        });
      for (const interfaceline of interfacelines) {
        if (interfaceline.has('functiontypeline')) {
          const functiontypeline = interfaceline.get('functiontypeline');
          const functionname = functiontypeline.get('variable').t;
          const typenames = [];
          typenames.push(
            functiontypeline.get('functiontype').get('fulltypename').t,
          );
          functiontypeline
            .get('functiontype')
            .get('cdr')
            .getAll()
            .forEach((r) => {
              typenames.push(r.get('fulltypename').t);
            });
          const returnType = scope.deepGet(
            functiontypeline.get('functiontype').get('returntype').t,
          ) as Type;
          if (!returnType || !(returnType instanceof Type)) {
            throw new Error(
              functiontypeline.get('functiontype').get('returntype').t +
                ' is not a type',
            );
          }
          const args = [];
          for (let i = 0; i < typenames.length; i++) {
            const argument = scope.deepGet(typenames[i]) as Type;
            if (!argument || !(argument instanceof Type)) {
              throw new Error(typenames[i] + ' is not a type');
            }
            args.push(argument);
          }
          const functionType = new FunctionType(functionname, args, returnType);
          iface.functionTypes.push(functionType);
        }
        if (interfaceline.has('operatortypeline')) {
          const operatorname = interfaceline
            .get('operatortypeline')
            .get('operators').t;
          const isPrefix = !interfaceline
            .get('operatortypeline')
            .has('optleftarg');
          const argTypenames = [];
          if (!isPrefix) {
            argTypenames.push(
              interfaceline
                .get('operatortypeline')
                .get('optleftarg')
                .get('leftarg').t,
            );
          }
          argTypenames.push(
            interfaceline.get('operatortypeline').get('rightarg').t,
          );
          const returnTypename = interfaceline
            .get('operatortypeline')
            .get('fulltypename').t;
          const args = argTypenames.map((n) => {
            const box = scope.deepGet(n);
            if (!box || !(box instanceof Type)) {
              throw new Error(`${n} is not a type`);
            }
            return box;
          });
          const returnType = scope.deepGet(returnTypename) as Type;
          if (!returnType || !(returnType instanceof Type)) {
            throw new Error(`${returnTypename} is not a type`);
          }
          const operatorType = new OperatorType(
            operatorname,
            isPrefix,
            args,
            returnType,
          );
          iface.operatorTypes.push(operatorType);
        }
        if (interfaceline.has('propertytypeline')) {
          const propertyType = scope.deepGet(
            interfaceline.get('propertytypeline').get('variable').t,
          ) as Type;
          if (!propertyType || !(propertyType instanceof Type)) {
            throw new Error(
              interfaceline.get('propertytypeline').get('variable').t +
                ' is not a type',
            );
          }
          iface.requiredProperties[
            interfaceline.get('propertytypeline').get('variable').t
          ] = propertyType;
        }
      }
    } else if (interfaceAst.get('interfacedef').has('interfacealias')) {
      const otherInterface = scope.deepGet(
        interfaceAst.get('interfacedef').get('interfacealias').get('variable')
          .t,
      ) as Type;
      if (!(otherInterface instanceof Type) || !otherInterface.iface) {
        throw new Error(
          `${
            interfaceAst
              .get('interfacedef')
              .get('interfacealias')
              .get('variable').t
          } is not an interface`,
        );
      }
      // Replace the interface with the other one
      ifaceType.iface = otherInterface.iface;
    }
    return ifaceType;
  }
}

export class Type {
  typename: string;
  builtIn: boolean;
  isGenericStandin: boolean;
  properties: Properties;
  generics: Generics;
  originalType: Type | null;
  iface: Interface | null;
  alias: Type | null;

  constructor(
    typename: string,
    builtIn = false,
    isGenericStandin = false,
    properties: Properties = {},
    generics: Generics = {},
    originalType: Type | null = null,
    iface: Interface | null = null,
    alias: Type | null = null,
  ) {
    this.typename = typename;
    this.builtIn = builtIn;
    this.isGenericStandin = isGenericStandin;
    this.properties = properties;
    this.generics = generics;
    this.originalType = originalType;
    this.iface = iface;
    this.alias = alias;
  }

  toString() {
    if (this.iface != null) return '// Interfaces TBD';
    let outString = 'type ' + this.typename;
    if (this.alias != null) {
      outString += ' = ' + this.alias.typename;
      return outString;
    }
    if (this.generics.length > 0) {
      outString += '<' + Object.keys(this.generics).join(', ') + '>';
    }
    outString += '{\n';
    for (const propName of Object.keys(this.properties)) {
      outString +=
        '  ' + propName + ': ' + this.properties[propName].typename + '\n';
    }
    outString += '}\n';
    return outString;
  }

  static fromAst(typeAst: LPNode, scope: Scope) {
    const type = new Type(typeAst.get('fulltypename').get('typename').t);
    const genScope = new Scope();
    const typeScope = new Scope(scope);
    typeScope.secondaryPar = genScope;
    if (typeAst.get('fulltypename').has('opttypegenerics')) {
      const genericsAst = typeAst
        .get('fulltypename')
        .get('opttypegenerics')
        .get('generics');
      const generics = [];
      generics.push(genericsAst.get('fulltypename').t);
      genericsAst
        .get('cdr')
        .getAll()
        .forEach((r) => {
          generics.push(r.get('fulltypename').t);
        });
      for (let i = 0; i < generics.length; i++) {
        type.generics[generics[i]] = i;
        genScope.put(generics[i], new Type(generics[i], true, true));
      }
    }
    if (typeAst.get('typedef').has('typebody')) {
      const typelist = typeAst.get('typedef').get('typebody').get('typelist');
      const lines = [];
      lines.push(typelist.get('typeline'));
      typelist
        .get('cdr')
        .getAll()
        .forEach((r) => {
          lines.push(r.get('typeline'));
        });
      for (const lineAst of lines) {
        const propertyName = lineAst.get('variable').t;
        const typeName = lineAst.get('fulltypename').t.trim();
        const property = typeScope.deepGet(typeName) as Type;
        if (!property || !(property instanceof Type)) {
          // Potentially a type that depends on the type generics of this type
          const baseTypeName = lineAst.get('fulltypename').get('typename').t;
          const genericsList = [];
          if (lineAst.get('fulltypename').has('opttypegenerics')) {
            const innerGenerics = lineAst
              .get('fulltypename')
              .get('opttypegenerics')
              .get('generics');
            genericsList.push(innerGenerics.get('fulltypename'));
            innerGenerics
              .get('cdr')
              .getAll()
              .forEach((r) => {
                genericsList.push(r.get('fulltypename'));
              });
          }
          const innerGenerics = [...genericsList];
          const genericsQueue = [];
          while (genericsList.length > 0) {
            const generic = genericsList.shift();
            genericsQueue.push(generic);
            if (generic.has('opttypegenerics')) {
              const innerInnerGenerics = generic
                .get('opttypegenerics')
                .get('generics');
              genericsList.push(innerInnerGenerics.get('fulltypename'));
              innerInnerGenerics
                .get('cdr')
                .getAll()
                .forEach((r) => {
                  genericsList.push(r.get('fulltypename'));
                });
            }
          }
          while (genericsQueue.length > 0) {
            const generic = genericsQueue.pop();
            const innerType = typeScope.deepGet(generic.t) as Type;
            if (!innerType) {
              const innerBaseTypeName = generic.get('typename').t;
              const innerBaseType = typeScope.deepGet(
                innerBaseTypeName,
              ) as Type;
              if (!innerBaseType) {
                throw new Error(
                  `Cannot find type ${innerBaseTypeName} while defining ${type}`,
                );
              }
              const innerBaseGenerics = [];
              if (generic.has('opttypegenerics')) {
                const innerInnerGenerics = generic
                  .get('opttypegenerics')
                  .get('generics');
                innerBaseGenerics.push(
                  innerInnerGenerics.get('fulltypename').t,
                );
                innerInnerGenerics
                  .get('cdr')
                  .getAll()
                  .forEach((r) => {
                    innerBaseGenerics.push(r.get('fulltypename').t);
                  });
              }
              innerBaseType.solidify(innerBaseGenerics, typeScope);
            }
          }
          const baseType = scope.deepGet(baseTypeName) as Type;
          if (!baseType || !(baseType instanceof Type)) {
            throw new Error(lineAst.get('fulltypename').t + ' is not a type');
          }
          type.properties[propertyName] = baseType.solidify(
            innerGenerics.map((r) => r.t),
            typeScope,
          );
        } else {
          type.properties[propertyName] = property;
        }
      }
    }
    if (typeAst.get('typedef').has('typealias')) {
      const otherType = scope.deepGet(
        typeAst
          .get('typedef')
          .get('typealias')
          .get('fulltypename')
          .get('typename').t,
      ) as Type;
      if (!otherType) {
        throw new Error(
          'Type ' +
            typeAst.get('typedef').get('typealias').get('fulltypename').t +
            ' not defined',
        );
      }
      if (!(otherType instanceof Type)) {
        throw new Error(
          typeAst.get('typedef').get('typealias').get('fulltypename').t +
            ' is not a valid type',
        );
      }

      let fulltypename = otherType;
      if (
        Object.keys(fulltypename.generics).length > 0 &&
        typeAst
          .get('typedef')
          .get('typealias')
          .get('fulltypename')
          .has('opttypegenerics')
      ) {
        const solidTypes = [];
        const innerTypeGenerics = typeAst
          .get('typedef')
          .get('typealias')
          .get('fulltypename')
          .get('opttypegenerics')
          .get('generics');
        solidTypes.push(innerTypeGenerics.get('fulltypename').t);
        innerTypeGenerics
          .get('cdr')
          .getAll()
          .forEach((r) => {
            solidTypes.push(r.get('fulltypename').t);
          });
        fulltypename = fulltypename.solidify(solidTypes, scope);
      }

      // For simplification of the type aliasing functionality, the other type is attached as
      // an alias. The module construction will, if present, perfer the alias over the actual
      // type, to make sure built-in types that are aliased continue to work. This means that
      // `type varA == type varB` will work if `varA` is assigned to an alias and `varB` to the
      // orignal type. I can see the argument either way on this, but the simplicity of this
      // approach is why I will go with this for now.
      type.alias = fulltypename;
    }
    scope.put(type.typename, type);
    return type;
  }

  solidify(genericReplacements: Array<string>, scope: Scope) {
    const genericTypes = Object.keys(this.generics).map(
      (t) => new Type(t, true, true),
    );
    const replacementTypes = [];
    for (const typename of genericReplacements) {
      const typebox = scope.deepGet(typename) as Type;
      if (!typebox || !(typebox instanceof Type)) {
        const fulltypename = fulltypenameAstFromString(typename);
        if (fulltypename.has('opttypegenerics')) {
          const basename = fulltypename.get('typename').t;
          const generics = [];
          generics.push(
            fulltypename
              .get('opttypegenerics')
              .get('generics')
              .get('fulltypename').t,
          );
          fulltypename
            .get('opttypegenerics')
            .get('generics')
            .get('cdr')
            .getAll()
            .forEach((r) => {
              generics.push(r.get('fulltypename').t);
            });
          const baseType = scope.deepGet(basename) as Type;
          if (!baseType || !(baseType instanceof Type)) {
            throw new Error(basename + ' type not found');
          } else {
            const newtype = baseType.solidify(generics, scope);
            replacementTypes.push(newtype);
          }
        } else {
          throw new Error(typename + ' type not found');
        }
      } else {
        replacementTypes.push(typebox);
      }
    }
    const genericMap = new Map();
    genericTypes.forEach((g, i) => genericMap.set(g, replacementTypes[i]));
    const solidifiedName =
      this.typename + '<' + genericReplacements.join(', ') + '>';
    const solidified = new Type(solidifiedName, this.builtIn);
    solidified.originalType = this;
    for (const propKey of Object.keys(this.properties)) {
      const propValue = this.properties[propKey];
      const newPropValue = propValue.realize(genericMap, scope);
      solidified.properties[propKey] = newPropValue;
    }
    scope.put(solidifiedName, solidified);
    return solidified;
  }

  typeApplies(
    otherType: Type,
    scope: Scope,
    interfaceMap: Map<Type, Type> = new Map(),
  ) {
    if (this.typename === otherType.typename) return true;
    if (this.iface) {
      const applies = this.iface.typeApplies(otherType, scope);
      if (applies) {
        interfaceMap.set(this, otherType);
      }
      return applies;
    }
    if (
      !this.originalType ||
      !otherType.originalType ||
      this.originalType.typename !== otherType.originalType.typename
    )
      return false;
    const typeAst = fulltypenameAstFromString(this.typename);
    const otherTypeAst = fulltypenameAstFromString(otherType.typename);
    let generics = [];
    if (typeAst.has('opttypegenerics')) {
      generics.push(
        typeAst.get('opttypegenerics').get('generics').get('fulltypename').t,
      );
      typeAst
        .get('opttypegenerics')
        .get('generics')
        .get('cdr')
        .getAll()
        .forEach((r) => {
          generics.push(r.get('fulltypename').t);
        });
    }
    generics = generics.map(
      (g) =>
        scope.deepGet(g) ||
        (Type.fromStringWithMap(g, interfaceMap, scope) as Type) ||
        new Type('-bogus-', false, true),
    );
    let otherGenerics = [];
    if (otherTypeAst.has('opttypegenerics')) {
      otherGenerics.push(
        otherTypeAst.get('opttypegenerics').get('generics').get('fulltypename')
          .t,
      );
      otherTypeAst
        .get('opttypegenerics')
        .get('generics')
        .get('cdr')
        .getAll()
        .forEach((r) => {
          otherGenerics.push(r.get('fulltypename').t);
        });
    }
    otherGenerics = otherGenerics.map(
      (g) =>
        scope.deepGet(g) ||
        (Type.fromStringWithMap(g, interfaceMap, scope) as Type) ||
        new Type('-bogus-', false, true),
    );
    return generics.every((t: Type, i) =>
      t.typeApplies(otherGenerics[i], scope, interfaceMap),
    );
  }

  hasInterfaceType(): boolean {
    if (this.iface) return true;
    return Object.values(this.properties).some((t: Type): boolean =>
      t.hasInterfaceType(),
    );
  }

  // There has to be a more elegant way to tackle this
  static fromStringWithMap(
    typestr: string,
    interfaceMap: Map<Type, Type>,
    scope: Scope,
  ) {
    const typeAst = fulltypenameAstFromString(typestr);
    const baseName = typeAst.get('typename').t;
    const baseType = scope.deepGet(baseName) as Type;
    if (typeAst.has('opttypegenerics')) {
      const genericNames = [];
      genericNames.push(
        typeAst.get('opttypegenerics').get('generics').get('fulltypename').t,
      );
      typeAst
        .get('opttypegenerics')
        .get('generics')
        .get('cdr')
        .getAll()
        .forEach((r) => {
          genericNames.push(r.get('fulltypename').t);
        });
      const generics = genericNames.map((t: string) => {
        const interfaceMapping = [...interfaceMap.entries()].find(
          (e) => e[0].typename === t.trim(),
        );
        if (interfaceMapping) return interfaceMapping[1];
        const innerType = Type.fromStringWithMap(t, interfaceMap, scope);
        return innerType;
      });
      return baseType.solidify(
        generics
          .map((g: Type) => interfaceMap.get(g) || g)
          .map((t: Type) => t.typename),
        scope,
      );
    } else {
      return interfaceMap.get(baseType) || baseType;
    }
  }

  realize(interfaceMap: Map<Type, Type>, scope: Scope) {
    if (this.isGenericStandin)
      return [...interfaceMap.entries()].find(
        (e) => e[0].typename === this.typename,
      )[1];
    if (!this.iface && !this.originalType) return this;
    if (this.iface) return interfaceMap.get(this) || this;
    const self = new Type(
      this.typename,
      this.builtIn,
      this.isGenericStandin,
      { ...this.properties },
      { ...this.generics },
      this.originalType,
      this.iface,
      this.alias,
    );
    const newProps = Object.values(self.properties).map((t) =>
      t.realize(interfaceMap, scope),
    );
    Object.keys(self.properties).forEach((k, i) => {
      self.properties[k] = newProps[i];
    });
    const newType = Type.fromStringWithMap(self.typename, interfaceMap, scope);
    return newType;
  }

  // This is only necessary for the numeric types. TODO: Can we eliminate it?
  castable(otherType: Type) {
    const intTypes = ['int8', 'int16', 'int32', 'int64'];
    const floatTypes = ['float32', 'float64'];
    if (
      intTypes.includes(this.typename) &&
      intTypes.includes(otherType.typename)
    )
      return true;
    if (
      floatTypes.includes(this.typename) &&
      floatTypes.includes(otherType.typename)
    )
      return true;
    if (
      floatTypes.includes(this.typename) &&
      intTypes.includes(otherType.typename)
    )
      return true;
    return false;
  }

  static builtinTypes = {
    void: new Type('void', true),
    int8: new Type('int8', true),
    int16: new Type('int16', true),
    int32: new Type('int32', true),
    int64: new Type('int64', true),
    float32: new Type('float32', true),
    float64: new Type('float64', true),
    bool: new Type('bool', true),
    string: new Type('string', true),
    Error: new Type('Error', true, false, {
      msg: new Type('string', true, true),
    }),
    Maybe: new Type(
      'Maybe',
      true,
      false,
      {
        value: new Type('T', true, true),
      },
      {
        T: 0,
      },
    ),
    Result: new Type(
      'Result',
      true,
      false,
      {
        value: new Type('T', true, true),
        error: new Type('Error', true, false, {
          msg: new Type('string', true, true),
        }),
      },
      {
        T: 0,
      },
    ),
    Either: new Type(
      'Either',
      true,
      false,
      {
        main: new Type('T', true, true),
        alt: new Type('U', true, true),
      },
      {
        T: 0,
        U: 1,
      },
    ),
    Array: new Type(
      'Array',
      true,
      false,
      {
        records: new Type('V', true, true),
      },
      {
        V: 0,
      },
    ),
    ExecRes: new Type('ExecRes', false, false, {
      exitCode: new Type('int64', true),
      stdout: new Type('string', true),
      stderr: new Type('string', true),
    }),
    InitialReduce: new Type(
      'InitialReduce',
      false,
      false,
      {
        arr: new Type(
          'Array<T>',
          true,
          false,
          {
            records: new Type('T', true, true),
          },
          {
            T: 0,
          },
        ),
        initial: new Type('U', true, true),
      },
      {
        T: 0,
        U: 1,
      },
    ),
    KeyVal: new Type(
      'KeyVal',
      false,
      false,
      {
        key: new Type('K', true, true),
        val: new Type('V', true, true),
      },
      {
        K: 0,
        V: 1,
      },
    ),
    // Placeholders to be replaced through weirdness with opcodes.ts as the self-referential piece
    // does not play well with `static`
    InternalRequest: new Type('InternalRequest', true, false, {
      method: new Type('string', true),
      url: new Type('string', true),
      headers: new Type('headers', true),
      body: new Type('string', true),
      connId: new Type('int64', true),
    }),
    InternalResponse: new Type('InternalResponse', true, false, {
      status: new Type('int64', true),
      headers: new Type('headers', true),
      body: new Type('string', true),
      connId: new Type('int64', true),
    }),
    Seq: new Type('Seq', true, false, {
      counter: new Type('int64', true, true),
      limit: new Type('int64', true, true),
    }),
    Self: new Type('Self', true, false, {
      seq: new Type('Seq', true, false, {
        counter: new Type('int64', true, true),
        limit: new Type('int64', true, true),
      }),
      recurseFn: new Type('function', true),
    }),
    TcpChannel: new Type('TcpChannel', true),
    TcpContext: new Type(
      'TcpContext',
      true,
      false,
      {
        context: new Type('C', true, true),
        channel: new Type('TcpChannel', true),
      },
      {
        C: 0,
      },
    ),
    Chunk: new Type('Chunk', true),
    function: new Type('function', true),
    operator: new Type('operator', true),
    Event: new Type(
      'Event',
      true,
      false,
      {
        type: new Type('E', true, true),
      },
      {
        E: 0,
      },
    ),
    type: new Type('type', true),
    scope: new Type('scope', true),
    microstatement: new Type('microstatement', true),
  };
}

export default Type;
