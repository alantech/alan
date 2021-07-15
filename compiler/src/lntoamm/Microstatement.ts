/* eslint-disable @typescript-eslint/no-var-requires */
import { v4 as uuid } from 'uuid';

import * as Ast from './Ast';
import Event from './Event';
import Operator from './Operator';
import Constant from './Constant';
import Scope from './Scope';
import Statement from './Statement';
import StatementType from './StatementType';
import Type from './Type';
import UserFunction from './UserFunction';
import { Args, Fn } from './Function';
import { LPNode } from '../lp';

const FIXED_TYPES = [
  'int64',
  'int32',
  'int16',
  'int8',
  'float64',
  'float32',
  'bool',
  'void',
];

class Microstatement {
  statementType: StatementType;
  scope: Scope;
  pure: boolean;
  outputName: string;
  alias: string;
  outputType: Type;
  inputNames: Array<string>;
  fns: Array<Fn>;
  closurePure: boolean;
  closureStatements: Array<Microstatement>;
  closureArgs: Args;
  closureOutputType: Type;

  constructor(
    statementType: StatementType,
    scope: Scope,
    pure: boolean,
    outputName: string,
    outputType: Type = Type.builtinTypes.void,
    inputNames: Array<string> = [],
    fns: Array<Fn> = [],
    alias = '',
    closurePure = true,
    closureStatements: Array<Microstatement> = [],
    closureArgs: Args = {},
    closureOutputType: Type = Type.builtinTypes.void,
  ) {
    this.statementType = statementType;
    this.scope = scope;
    this.pure = pure;
    this.outputName = outputName;
    this.outputType = outputType;
    this.inputNames = inputNames;
    this.fns = fns;
    this.alias = alias;
    this.closurePure = closurePure;
    this.closureStatements = closureStatements;
    this.closureArgs = closureArgs;
    this.closureOutputType = closureOutputType;
  }

  toString() {
    let outString = '';
    switch (this.statementType) {
      case StatementType.CONSTDEC:
        outString =
          'const ' + this.outputName + ': ' + this.outputType.typename;
        if (this.fns.length > 0) {
          outString +=
            ' = ' +
            this.fns[0].getName() +
            '(' +
            this.inputNames.join(', ') +
            ')';
        } else if (this.inputNames.length > 0) {
          outString += ' = ' + this.inputNames[0]; // Doesn't appear the list is ever used here
        }
        break;
      case StatementType.LETDEC:
        outString = 'let ' + this.outputName + ': ' + this.outputType.typename;
        if (this.fns.length > 0) {
          outString +=
            ' = ' +
            this.fns[0].getName() +
            '(' +
            this.inputNames.join(', ') +
            ')';
        } else if (this.inputNames.length > 0) {
          outString += ' = ' + this.inputNames[0]; // Doesn't appear the list is ever used here
        }
        break;
      case StatementType.ASSIGNMENT:
        outString = this.outputName;
        if (this.fns.length > 0) {
          outString +=
            ' = ' +
            this.fns[0].getName() +
            '(' +
            this.inputNames.join(', ') +
            ')';
        } else if (this.inputNames.length > 0) {
          outString += ' = ' + this.inputNames[0]; // Doesn't appear the list is ever used here
        } else {
          outString += 'NO!';
        }
        break;
      case StatementType.CALL:
        if (this.fns.length > 0) {
          outString +=
            this.fns[0].getName() + '(' + this.inputNames.join(', ') + ')';
        }
        break;
      case StatementType.EMIT:
        outString = 'emit ' + this.outputName + ' ';
        if (this.fns.length > 0) {
          outString +=
            this.fns[0].getName() + '(' + this.inputNames.join(', ') + ')';
        } else if (this.inputNames.length > 0) {
          outString += this.inputNames[0]; // Doesn't appear the list is ever used here
        }
        break;
      case StatementType.EXIT:
        outString = 'return ' + this.outputName;
        break;
      case StatementType.CLOSURE:
        outString = 'const ' + this.outputName + ': function = fn (';
        const args = [];
        for (const [name, type] of Object.entries(this.closureArgs)) {
          if (name !== '' && type.typename != '') {
            args.push(name + ': ' + type.typename);
          }
        }
        outString += args.join(',');
        outString += '): ' + this.closureOutputType.typename + ' {\n';
        for (const m of this.closureStatements) {
          const s = m.toString();
          if (s !== '') {
            outString += '    ' + m.toString() + '\n';
          }
        }
        outString += '  }';
        break;
      case StatementType.REREF:
      case StatementType.ARG:
      case StatementType.CLOSUREDEF:
        // Intentionally never output anything, this is metadata for the transpiler algo only
        break;
    }
    return outString;
  }

  static fromVarName(
    varName: string,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    let original = null;
    for (let i = microstatements.length - 1; i > -1; i--) {
      const microstatement = microstatements[i];
      // TODO: var resolution is complex. Need to revisit this.
      if (microstatement.outputName === varName) {
        original = microstatement;
        if (microstatement.statementType !== StatementType.REREF) {
          break;
        }
      }
      if (microstatement.alias === varName) {
        original = microstatement;
        for (let j = i - 1; j >= 0; j--) {
          if (
            microstatements[j].outputName === original.outputName &&
            microstatements[j].statementType !== StatementType.REREF
          ) {
            original = microstatements[j];
            break;
          }
        }
        break;
      }
    }
    // Check if this is a module constant that should be un-hoisted
    if (
      original === null &&
      !!scope.deepGet(varName) &&
      scope.deepGet(varName) instanceof Constant
    ) {
      const globalConst = scope.deepGet(varName) as Constant;
      Microstatement.fromAssignablesAst(
        globalConst.assignablesAst,
        globalConst.scope, // Eval this in its original scope in case it was an exported const
        microstatements, // that was dependent on unexported internal functions or constants
      );
      const last = microstatements[microstatements.length - 1];
      microstatements.push(
        new Microstatement(
          StatementType.REREF,
          scope,
          true,
          last.outputName,
          last.outputType,
          [],
          [],
          globalConst.name,
        ),
      );
    }
    return original;
  }

  static fromConstantsAst(
    constantsAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const constName = '_' + uuid().replace(/-/g, '_');
    let constType = 'void';
    if (constantsAst.has('bool')) constType = 'bool';
    if (constantsAst.has('str')) constType = 'string';
    if (constantsAst.has('num')) {
      // TODO: Add support for hex, octal, scientific, etc
      const numberConst = constantsAst.t;
      constType = numberConst.indexOf('.') > -1 ? 'float64' : 'int64';
    }
    let constVal: string;
    try {
      JSON.parse(constantsAst.t); // Will fail on strings with escape chars
      constVal = constantsAst.t;
    } catch (e) {
      // It may be a zero-padded number
      if (
        ['int8', 'int16', 'int32', 'int64'].includes(constType) &&
        constantsAst.t[0] === '0'
      ) {
        constVal = parseInt(constantsAst.t, 10).toString();
      } else if (
        ['float32', 'float64'].includes(constType) &&
        constantsAst.t[0] === '0'
      ) {
        constVal = parseFloat(constantsAst.t).toString();
      } else {
        // Hackery to get these strings to work
        constVal = JSON.stringify(
          constantsAst.t.replace(/^["']/, '').replace(/["']$/, ''),
        );
      }
    }
    microstatements.push(
      new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        constName,
        scope.deepGet(constType) as Type,
        [constVal],
        [],
      ),
    );
  }

  static fromObjectLiteralsAst(
    objectLiteralsAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    if (objectLiteralsAst.has('arrayliteral')) {
      // Array literals first need all of the microstatements of the array contents defined, then
      // a `newarr` opcode call is inserted for the object literal itself, then `pusharr` opcode
      // calls are emitted to insert the relevant data into the array, and finally the array itself
      // is REREFed for the outer microstatement generation call.
      let arrayLiteralContents = [];
      const arraybase = objectLiteralsAst.get('arrayliteral').has('arraybase')
        ? objectLiteralsAst.get('arrayliteral').get('arraybase')
        : objectLiteralsAst
            .get('arrayliteral')
            .get('fullarrayliteral')
            .get('arraybase');
      if (arraybase.has('assignablelist')) {
        const assignablelist = arraybase.get('assignablelist');
        arrayLiteralContents.push(assignablelist.get('assignables'));
        assignablelist
          .get('cdr')
          .getAll()
          .forEach((r) => {
            arrayLiteralContents.push(r.get('assignables'));
          });
        arrayLiteralContents = arrayLiteralContents.map((r) => {
          Microstatement.fromAssignablesAst(r, scope, microstatements);
          return microstatements[microstatements.length - 1];
        });
      }
      let type = null;
      if (objectLiteralsAst.get('arrayliteral').has('fullarrayliteral')) {
        const arrayTypeAst = objectLiteralsAst
          .get('arrayliteral')
          .get('fullarrayliteral')
          .get('literaldec')
          .get('fulltypename');
        type = scope.deepGet(arrayTypeAst.t.trim()) as Type;
        if (!type) {
          // Try to define it if it's a generic type
          if (arrayTypeAst.has('opttypegenerics')) {
            const outerType = scope.deepGet(
              arrayTypeAst.get('typename').t.trim(),
            ) as Type;
            if (!outerType) {
              throw new Error(`${arrayTypeAst.t}  is not defined
${objectLiteralsAst.t} on line ${objectLiteralsAst.line}:${objectLiteralsAst.char}`);
            }
            const generics = [];
            const genericsAst = arrayTypeAst
              .get('opttypegenerics')
              .get('generics');
            generics.push(genericsAst.get('fulltypename').t);
            genericsAst
              .get('cdr')
              .getAll()
              .forEach((r) => {
                generics.push(r.get('fulltypename').t);
              });
            outerType.solidify(generics, scope);
            type = scope.deepGet(arrayTypeAst.t.trim());
          }
        }
        if (!(type instanceof Type)) {
          throw new Error(`${arrayTypeAst.t.trim()} is not a type
${objectLiteralsAst.t} on line ${objectLiteralsAst.line}:${
            objectLiteralsAst.char
          }`);
        }
      } else if (arrayLiteralContents.length > 0) {
        const innerType = arrayLiteralContents[0].outputType.typename;
        Type.builtinTypes['Array'].solidify([innerType], scope);
        type = scope.deepGet(`Array<${innerType}>`) as Type;
      } else {
        throw new Error(`Ambiguous array type, please specify the type for an empty array with the syntax \`new Array<MyType> []\`
${objectLiteralsAst.t} on line ${objectLiteralsAst.line}:${objectLiteralsAst.char}`);
      }
      // Create a new variable to hold the size of the array literal
      const lenName = '_' + uuid().replace(/-/g, '_');
      microstatements.push(
        new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          lenName,
          Type.builtinTypes['int64'],
          [`${arrayLiteralContents.length}`],
          [],
        ),
      );
      // Add the opcode to create a new array with the specified size
      const opcodes = require('./opcodes').default;
      opcodes.exportScope
        .get('newarr')[0]
        .microstatementInlining([lenName], scope, microstatements);
      // Get the array microstatement and extract the name and insert the correct type
      const array = microstatements[microstatements.length - 1];
      array.outputType = type;
      // Try to use the "real" type if knowable
      if (arrayLiteralContents.length > 0) {
        array.outputType = Type.builtinTypes['Array'].solidify(
          [arrayLiteralContents[0].outputType.typename],
          scope,
        );
      }
      const arrayName = array.outputName;
      // Push the values into the array
      for (let i = 0; i < arrayLiteralContents.length; i++) {
        // Create a new variable to hold the size of the array value
        const size = FIXED_TYPES.includes(
          arrayLiteralContents[i].outputType.typename,
        )
          ? '8'
          : '0';
        const sizeName = '_' + uuid().replace(/-/g, '_');
        microstatements.push(
          new Microstatement(
            StatementType.CONSTDEC,
            scope,
            true,
            sizeName,
            Type.builtinTypes['int64'],
            [size],
            [],
          ),
        );
        // Push the value into the array
        const opcodes = require('./opcodes').default;
        opcodes.exportScope
          .get('pusharr')[0]
          .microstatementInlining(
            [arrayName, arrayLiteralContents[i].outputName, sizeName],
            scope,
            microstatements,
          );
      }
      // REREF the array
      microstatements.push(
        new Microstatement(
          StatementType.REREF,
          scope,
          true,
          arrayName,
          array.outputType,
          [],
          [],
        ),
      );
    } else if (objectLiteralsAst.has('typeliteral')) {
      // User types are represented in AMM and lower as `Array<any>`. This reduces the number of
      // concepts that have to be maintained in the execution layer (and is really what C structs
      // are, anyways). The order of the properties on the specified type directly map to the
      // order that they are inserted into the Array, not the order they're defined in the object
      // literal notation, so reads and updates later on can occur predictably by mapping the name
      // of the property to its array index.
      //
      // If the type literal is missing any fields, that's a hard compile error to make sure
      // accessing undefined data is impossible. If a value might not be needed, they should use
      // the `Option` type and provide a `None` value there.
      const typeAst = objectLiteralsAst
        .get('typeliteral')
        .get('literaldec')
        .get('fulltypename');
      let type = scope.deepGet(typeAst.t.trim()) as Type;
      if (type === null) {
        // Try to define it if it's a generic type
        if (typeAst.has('opttypegenerics')) {
          const outerType = scope.deepGet(
            typeAst.get('typename').t.trim(),
          ) as Type;
          if (outerType === null) {
            throw new Error(`${typeAst.t} is not defined
${objectLiteralsAst.t} on line ${objectLiteralsAst.line}:${objectLiteralsAst.char}`);
          }
          const generics = [];
          const genericsAst = typeAst.get('opttypegenerics').get('generics');
          generics.push(genericsAst.get('fulltypename').t);
          genericsAst
            .get('cdr')
            .getAll()
            .forEach((r) => {
              generics.push(r.get('fulltypename').t);
            });
          outerType.solidify(generics, scope);
          type = scope.deepGet(typeAst.t.trim()) as Type;
        }
      }
      if (!(type instanceof Type)) {
        throw new Error(`${typeAst.t.trim()} is not a type
${objectLiteralsAst.t} on line ${objectLiteralsAst.line}:${
          objectLiteralsAst.char
        }`);
      }
      const assignlist = objectLiteralsAst
        .get('typeliteral')
        .get('typebase')
        .get('typeassignlist');
      const assignArr = [];
      assignArr.push({
        field: assignlist.get('variable'),
        val: assignlist.get('assignables'),
      });
      assignlist
        .get('cdr')
        .getAll()
        .forEach((r) => {
          assignArr.push({
            field: r.get('variable'),
            val: r.get('assignables'),
          });
        });
      const assignfields = assignArr.map((r) => r.field.t);
      const assignvals = assignArr.map((r) => r.val);
      const fields = Object.keys(type.properties);
      const missingFields = [];
      const foundFields = [];
      const extraFields = [];
      const astLookup = {};
      for (let i = 0; i < assignfields.length; i++) {
        const assignfield = assignfields[i];
        const assignval = assignvals[i];
        astLookup[assignfield] = assignval;
        if (!fields.includes(assignfield)) {
          extraFields.push(assignfield);
        }
        if (foundFields.includes(assignfield)) {
          extraFields.push(assignfield);
        }
        foundFields.push(assignfield);
      }
      for (const field of fields) {
        if (!foundFields.includes(field)) {
          missingFields.push(field);
        }
      }
      if (missingFields.length > 0 || extraFields.length > 0) {
        let errMsg = `${typeAst.t.trim()} object literal improperly defined`;
        if (missingFields.length > 0) {
          errMsg += '\n' + `Missing fields: ${missingFields.join(', ')}`;
        }
        if (extraFields.length > 0) {
          errMsg += '\n' + `Extra fields: ${extraFields.join(', ')}`;
        }
        errMsg +=
          '\n' +
          objectLiteralsAst.t +
          ' on line ' +
          objectLiteralsAst.line +
          ':' +
          objectLiteralsAst.char;
        throw new Error(errMsg);
      }
      // The assignment looks good, now we'll mimic the array literal logic mostly
      const arrayLiteralContents = [];
      for (let i = 0; i < fields.length; i++) {
        Microstatement.fromAssignablesAst(
          astLookup[fields[i]],
          scope,
          microstatements,
        );
        arrayLiteralContents.push(microstatements[microstatements.length - 1]);
      }
      // Create a new variable to hold the size of the array literal
      const lenName = '_' + uuid().replace(/-/g, '_');
      microstatements.push(
        new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          lenName,
          Type.builtinTypes['int64'],
          [`${fields.length}`],
          [],
        ),
      );
      // Add the opcode to create a new array with the specified size
      const opcodes = require('./opcodes').default;
      opcodes.exportScope
        .get('newarr')[0]
        .microstatementInlining([lenName], scope, microstatements);
      // Get the array microstatement and extract the name and insert the correct type
      const array = microstatements[microstatements.length - 1];
      array.outputType = type;
      const arrayName = array.outputName;
      // Push the values into the array
      for (let i = 0; i < arrayLiteralContents.length; i++) {
        // Create a new variable to hold the size of the array value
        const size = FIXED_TYPES.includes(
          arrayLiteralContents[i].outputType.typename,
        )
          ? '8'
          : '0';
        const sizeName = '_' + uuid().replace(/-/g, '_');
        microstatements.push(
          new Microstatement(
            StatementType.CONSTDEC,
            scope,
            true,
            sizeName,
            Type.builtinTypes['int64'],
            [size],
            [],
          ),
        );
        // Push the value into the array
        const opcodes = require('./opcodes').default;
        opcodes.exportScope
          .get('pusharr')[0]
          .microstatementInlining(
            [arrayName, arrayLiteralContents[i].outputName, sizeName],
            scope,
            microstatements,
          );
      }
      // REREF the array
      microstatements.push(
        new Microstatement(
          StatementType.REREF,
          scope,
          true,
          arrayName,
          array.outputType,
          [],
          [],
        ),
      );
    }
  }

  static closureDef(
    fns: Array<Fn>,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const closuredefName = '_' + uuid().replace(/-/g, '_');
    // Keep any rerefs around as closure references
    const rerefs = microstatements.filter(
      (m) => m.statementType === StatementType.REREF,
    );
    microstatements.push(
      new Microstatement(
        StatementType.CLOSUREDEF,
        scope,
        true, // TODO: What should this be?
        closuredefName,
        Type.builtinTypes['function'],
        [],
        fns,
        '',
        true,
        rerefs,
      ),
    );
  }

  static closureFromUserFunction(
    userFunction: UserFunction,
    scope: Scope,
    microstatements: Array<Microstatement>,
    interfaceMap: Map<Type, Type>,
  ) {
    const fn = userFunction.maybeTransform(interfaceMap);
    const idx = microstatements.length;
    const args = Object.entries(fn.args);
    for (const [name, type] of args) {
      if (name !== '' && type.typename != '') {
        microstatements.push(
          new Microstatement(StatementType.LETDEC, scope, true, name, type),
        );
      }
    }
    const len = microstatements.length - args.length;
    for (const s of fn.statements) {
      Microstatement.fromStatementsAst(s.statementAst, scope, microstatements);
    }
    microstatements.splice(idx, args.length);
    const newlen = microstatements.length;
    // There might be off-by-one bugs in the conversion here
    const innerMicrostatements = microstatements.slice(len, newlen);
    microstatements.splice(len, newlen - len);
    const constName = '_' + uuid().replace(/-/g, '_');
    // if closure is not void return the last inner statement
    // TODO: Revisit this, if the closure doesn't have a type defined, sometimes it can only be
    // determined in the calling context and shouldn't be assumed to be `void`
    if (
      innerMicrostatements.length > 0 &&
      fn.getReturnType() !== Type.builtinTypes.void
    ) {
      const last = innerMicrostatements[innerMicrostatements.length - 1];
      innerMicrostatements.push(
        new Microstatement(
          StatementType.EXIT,
          scope,
          true,
          last.outputName,
          last.outputType,
        ),
      );
    }
    microstatements.push(
      new Microstatement(
        StatementType.CLOSURE,
        scope,
        true, // TODO: Figure out if this is true or not
        constName,
        Type.builtinTypes['function'],
        [],
        [],
        '',
        fn.pure,
        innerMicrostatements,
        fn.args,
        fn.getReturnType(),
      ),
    );
  }

  static fromEmitsAst(
    emitsAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    if (emitsAst.get('retval').has()) {
      // If there's an assignable value here, add it to the list of microstatements first, then
      // rewrite the final const assignment as the emit statement.
      Microstatement.fromAssignablesAst(
        emitsAst.get('retval').get('assignables'),
        scope,
        microstatements,
      );
      const event = scope.deepGet(emitsAst.get('eventname').t);
      if (!(event instanceof Event)) {
        throw new Error(`${emitsAst.get('eventname').t} is not an event!
${emitsAst.t} on line ${emitsAst.line}:${emitsAst.char}`);
      }
      const last = microstatements[microstatements.length - 1];
      if (
        last.outputType != event.type &&
        !event.type.castable(last.outputType)
      ) {
        throw new Error(`Attempting to assign a value of type ${last.outputType.typename} to an event of type ${event.type.typename}
${emitsAst.t} on line ${emitsAst.line}:${emitsAst.char}`);
      }
      microstatements.push(
        new Microstatement(
          StatementType.EMIT,
          scope,
          true,
          event.name,
          event.type,
          [last.outputName],
          [],
        ),
      );
    } else {
      // Otherwise, create an emit statement with no value
      const event = scope.deepGet(emitsAst.get('eventname').t) as Event;
      if (!(event instanceof Event)) {
        throw new Error(`${emitsAst.get('eventname').t} is not an event!
${emitsAst.t} on line ${emitsAst.line}:${emitsAst.char}`);
      }
      if (event.type != Type.builtinTypes.void) {
        throw new Error(`${emitsAst.get('eventname').t} must have a ${
          event.type
        } value emitted to it!
${emitsAst.t} on line ${emitsAst.line}:${emitsAst.char}`);
      }
      microstatements.push(
        new Microstatement(
          StatementType.EMIT,
          scope,
          true,
          event.name,
          Type.builtinTypes.void,
          [],
          [],
        ),
      );
    }
  }

  static fromExitsAst(
    exitsAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // `alan--` handlers don't have the concept of a `return` statement, the functions are all inlined
    // and the last assigned value for the function *is* the return statement
    if (exitsAst.get('retval').has()) {
      // If there's an assignable value here, add it to the list of microstatements
      Microstatement.fromAssignablesAst(
        exitsAst.get('retval').get('assignables'),
        scope,
        microstatements,
      );
    } else {
      // Otherwise, create a microstatement with no value
      const constName = '_' + uuid().replace(/-/g, '_');
      microstatements.push(
        new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          constName,
          Type.builtinTypes.void,
          ['void'],
          null,
        ),
      );
    }
  }

  static fromAssignmentsAst(
    assignmentsAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // For reassigning to a variable, we need to determine that the root variable is a
    // `let`-defined mutable variable and then tease out if any array or property accesses are done,
    // and if so we need to `register` a mutable reference to the array memory space and then update
    // the value with a `register` call from the assignables result address to the relevant inner
    // address of the last access argument. The format of a `varn` can only be the following:
    // `{moduleScope}.varName[arrayAccess].userProperty` where the array accesses and userProperties
    // can come in any order after the preamble. *Fortunately,* for this scenario, any situation
    // where `moduleScope` is included is invalid since only constants can be exported out of a
    // module, not mutable values, so we only need to read the *first* segment to immediately
    // determine if it is relevant or not -- if it comes back as a `Scope` object we abort with an
    // error. If not, then we find the relevant `Microstatement` and determine if it is a `const`
    // or a `let` declaration and abort if it is a `const`. After that, if there are no segments
    // beyond the first one, we simply take the `assignable` microstatement output and turn it into
    // an `ASSIGNMENT` StatementType, otherwise we need to go through a more complicated procedure
    // to `register` the `n-1` remaining inner array segments to new variables as references and
    // finally `register` the `assignable` into the location the last segment indicates.
    const segments = assignmentsAst.get('varn').getAll();
    // Now, find the original variable and confirm that it actually is a let declaration
    const letName = segments[0].t;
    let actualLetName: string;
    let original: Microstatement;
    for (let i = microstatements.length - 1; i >= 0; i--) {
      const microstatement = microstatements[i];
      if (microstatement.alias === letName) {
        actualLetName = microstatement.outputName;
        continue;
      }
      if (microstatement.outputName === actualLetName) {
        if (microstatement.statementType === StatementType.LETDEC) {
          original = microstatement;
          break;
        } else if (microstatement.statementType === StatementType.REREF) {
          original = Microstatement.fromVarName(
            microstatement.outputName,
            scope,
            microstatements,
          );
          break;
        } else if (microstatement.statementType === StatementType.ASSIGNMENT) {
          // We could treat this as evidence that it's cool, but let's just skip it.
          continue;
        } else {
          throw new Error(`Attempting to reassign a non-let variable.
${assignmentsAst.t} on line ${assignmentsAst.line}:${assignmentsAst.char}`);
        }
      }
      if (microstatement.outputName === letName) {
        original = microstatement;
      }
    }
    if (!original) {
      throw new Error(`Attempting to reassign to an undeclared variable
${assignmentsAst.t} on line ${assignmentsAst.line}:${assignmentsAst.char}`);
    }
    if (segments.length === 1) {
      // Could be a simple let variable
      const letName = segments[0].t;
      let actualLetName: string;
      for (let i = microstatements.length - 1; i >= 0; i--) {
        const microstatement = microstatements[i];
        if (microstatement.alias === letName) {
          actualLetName = microstatement.outputName;
          continue;
        }
        if (microstatement.outputName === actualLetName) {
          if (microstatement.statementType === StatementType.LETDEC) {
            break;
          } else if (microstatement.statementType === StatementType.REREF) {
            original = Microstatement.fromVarName(
              microstatement.outputName,
              scope,
              microstatements,
            );
            break;
          } else if (
            microstatement.statementType === StatementType.ASSIGNMENT
          ) {
            // Could treat this as evidence that it's okay, but let's be sure about that
            continue;
          } else {
            throw new Error(`Attempting to reassign a non-let variable.
${letName} on line ${assignmentsAst.line}:${assignmentsAst.char}`);
          }
        }
        if (microstatement.outputName === letName) {
          actualLetName = letName;
        }
      }
      Microstatement.fromAssignablesAst(
        assignmentsAst.get('assignables'),
        scope,
        microstatements,
      );
      // By definition the last microstatement is the const assignment we care about, so we can
      // just mutate its object to rename the output variable name to the name we need instead.
      let last = microstatements[microstatements.length - 1];
      if (last.statementType === StatementType.REREF) {
        // Find what it's rereferencing and adjust that, instead
        for (let i = microstatements.length - 2; i >= 0; i--) {
          const m = microstatements[i];
          if (
            m.outputName === last.outputName &&
            m.statementType !== StatementType.REREF
          ) {
            last = m;
            break;
          }
        }
      }
      if (last.statementType === StatementType.LETDEC) {
        // Insert a ref call for this instead of mutating the original assignment
        Microstatement.fromAssignablesAst(
          Ast.assignablesAstFromString(`ref(${last.outputName})`),
          scope,
          microstatements,
        );
        last = microstatements[microstatements.length - 1];
        if (last.statementType === StatementType.REREF) {
          // Find what it's rereferencing and adjust that, instead
          for (let i = microstatements.length - 2; i >= 0; i--) {
            const m = microstatements[i];
            if (
              m.outputName === last.outputName &&
              m.statementType !== StatementType.REREF
            ) {
              last = m;
              break;
            }
          }
        }
      }
      last.outputName = actualLetName;
      last.statementType = StatementType.ASSIGNMENT;
      // Attempt to "merge" the output types, useful for multiple branches assigning into the same
      // variable but only part of the type information is known in each branch (like in `Result`
      // or `Either` with the result value only in one branch or one type in each of the branches
      // for `Either`).
      if (original.outputType.typename !== last.outputType.typename) {
        if (original.outputType.iface) {
          // Just overwrite if it's an interface type
          original.outputType = last.outputType;
        } else if (
          !!original.outputType.originalType &&
          !!last.outputType.originalType &&
          original.outputType.originalType.typename ===
            last.outputType.originalType.typename
        ) {
          // The tricky path, let's try to merge the two types together
          const baseType = original.outputType.originalType;
          const originalTypeAst = Ast.fulltypenameAstFromString(
            original.outputType.typename,
          );
          const lastTypeAst = Ast.fulltypenameAstFromString(
            last.outputType.typename,
          );
          const originalSubtypes = [];
          if (originalTypeAst.has('opttypegenerics')) {
            const originalTypeGenerics = originalTypeAst
              .get('opttypegenerics')
              .get('generics');
            originalSubtypes.push(originalTypeGenerics.get('fulltypename').t);
            originalTypeGenerics
              .get('cdr')
              .getAll()
              .forEach((r) => {
                originalSubtypes.push(r.get('fulltypename').t);
              });
          }
          const lastSubtypes = [];
          if (lastTypeAst.has('opttypegenerics')) {
            const lastTypeGenerics = lastTypeAst
              .get('opttypegenerics')
              .get('generics');
            lastSubtypes.push(lastTypeGenerics.get('fulltypename').t);
            lastTypeGenerics
              .get('cdr')
              .getAll()
              .forEach((r) => {
                lastSubtypes.push(r.get('fulltypename').t);
              });
          }
          const newSubtypes = [];
          for (let i = 0; i < originalSubtypes.length; i++) {
            if (originalSubtypes[i] === lastSubtypes[i]) {
              newSubtypes.push(originalSubtypes[i]);
            } else {
              const originalSubtype = scope.deepGet(
                originalSubtypes[i],
              ) as Type;
              if (originalSubtype.iface) {
                newSubtypes.push(lastSubtypes[i]);
              } else if (originalSubtype.originalType) {
                // TODO: Support nesting
                newSubtypes.push(originalSubtypes[i]);
              } else {
                newSubtypes.push(originalSubtypes[i]);
              }
            }
          }
          const newType = baseType.solidify(newSubtypes, scope);
          original.outputType = newType;
        } else {
          // Hmm... what to do here?
          original.outputType = last.outputType;
        }
      }
      return;
    }
    // The more complicated path. First, rule out that the first segment is not a `scope`.
    const test = scope.deepGet(segments[0].t);
    if (!!test && test instanceof Scope) {
      throw new Error(`Atempting to reassign to variable from another module
${assignmentsAst.get('varn').t} on line ${assignmentsAst.line}:${
        assignmentsAst.char
      }`);
    }
    let nestedLetType = original.outputType;
    for (let i = 1; i < segments.length - 1; i++) {
      const segment = segments[i];
      // A separator, just do nothing else this loop
      if (segment.has('methodsep')) continue;
      // An array access. Until the grammar definition is reworked, this will parse correctly, but
      // it is banned in alan (due to being unable to catch and report assignment errors to arrays)
      if (segment.has('arrayaccess')) {
        throw new Error(
          `${segments.join(
            '',
          )} cannot be written to. Please use 'set' to mutate arrays and hash tables`,
        );
      }
      // If it's a varname here, then we're accessing an inner property type. We need to figure out
      // which index it is in the underlying array structure and then `register` that piece (since
      // this is an intermediate access and not the final access point)
      if (segment.has('variable')) {
        const fieldName = segment.get('variable').t;
        const fields = Object.keys(nestedLetType.properties);
        const fieldNum = fields.indexOf(fieldName);
        if (fieldNum < 0) {
          // Invalid object access
          throw new Error(`${letName} does not have a field named ${fieldName}
${assignmentsAst.get('varn').t} on line ${assignmentsAst.get('varn').line}:${
            assignmentsAst.get('varn').char
          }`);
        }
        // Create a new variable to hold the address within the array literal
        const addrName = '_' + uuid().replace(/-/g, '_');
        microstatements.push(
          new Microstatement(
            StatementType.CONSTDEC,
            scope,
            true,
            addrName,
            Type.builtinTypes['int64'],
            [`${fieldNum}`],
            [],
          ),
        );
        // Insert a `register` opcode.
        const opcodes = require('./opcodes').default;
        opcodes.exportScope
          .get('register')[0]
          .microstatementInlining(
            [original.outputName, addrName],
            scope,
            microstatements,
          );
        // Now, we need to update the type we're working with.
        nestedLetType = Object.values(nestedLetType.properties)[fieldNum];
        // Now update the `original` record to the new `register` result
        original = microstatements[microstatements.length - 1];
      }
    }
    Microstatement.fromAssignablesAst(
      assignmentsAst.get('assignables'),
      scope,
      microstatements,
    );
    // Grab a reference to the final assignment variable.
    const assign = microstatements[microstatements.length - 1];
    // Next, determine which kind of final segment this is and perform the appropriate action to
    // insert into with a `copytof` or `copytov` opcode.
    const copytoop = [
      'int8',
      'int16',
      'int32',
      'int64',
      'float32',
      'float64',
      'bool',
    ].includes(assign.outputType.typename)
      ? 'copytof'
      : 'copytov';
    const finalSegment = segments[segments.length - 1];
    if (finalSegment.has('variable')) {
      const fieldName = finalSegment.t;
      const fields = Object.keys(nestedLetType.properties);
      const fieldNum = fields.indexOf(fieldName);
      if (fieldNum < 0) {
        // Invalid object access
        throw new Error(`${letName} does not have a field named ${fieldName}
${letName} on line ${assignmentsAst.line}:${assignmentsAst.char}`);
      }
      // Check if the new variable is allowed to be assigned to this object
      const originalType = nestedLetType.properties[fieldName];
      if (!originalType.typeApplies(assign.outputType, scope)) {
        throw new Error(
          `${letName}.${fieldName} is of type ${originalType.typename} but assigned a value of type ${assign.outputType.typename}`,
        );
      }
      // Create a new variable to hold the address within the array literal
      const addrName = '_' + uuid().replace(/-/g, '_');
      microstatements.push(
        new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          addrName,
          Type.builtinTypes['int64'],
          [`${fieldNum}`],
          [],
        ),
      );
      // Insert a `copytof` or `copytov` opcode.
      const opcodes = require('./opcodes').default;
      opcodes.exportScope
        .get(copytoop)[0]
        .microstatementInlining(
          [original.outputName, addrName, assign.outputName],
          scope,
          microstatements,
        );
    } else {
      throw new Error(`${finalSegment.t} cannot be the final piece in a reassignment statement
${letName} on line ${assignmentsAst.line}:${assignmentsAst.char}`);
    }
  }

  static fromLetdeclarationAst(
    letdeclarationAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const letAlias = letdeclarationAst.get('variable').t;
    const letTypeHint = letdeclarationAst.get('typedec').has()
      ? letdeclarationAst.get('typedec').get('fulltypename').t
      : '';
    const type = scope.deepGet(letTypeHint);
    if (type === null && letTypeHint !== '') {
      // Try to define it if it's a generic type
      const letTypeAst = letdeclarationAst.get('typedec').get('fulltypename');
      if (letTypeAst.has('opttypegenerics')) {
        const outerType = scope.deepGet(letTypeAst.get('typename').t) as Type;
        if (outerType === null) {
          throw new Error(`${letTypeAst.get('typename').t}  is not defined
${letdeclarationAst.t} on line ${letdeclarationAst.line}:${
            letdeclarationAst.char
          }`);
        }
        const generics = [];
        const genericAst = letTypeAst.get('opttypegenerics').get('generics');
        generics.push(genericAst.get('fulltypename').t);
        genericAst
          .get('cdr')
          .getAll()
          .forEach((r) => {
            generics.push(r.get('fulltypename').t);
          });
        outerType.solidify(generics, scope);
      }
    }
    Microstatement.fromAssignablesAst(
      letdeclarationAst.get('assignables'),
      scope,
      microstatements,
    );
    // By definition the last microstatement is the const assignment we care about, so we can just
    // mutate its object to rename the output variable name to the name we need instead.
    // EXCEPT with Arrays and User Types. The last is a REREF, so follow it back to the original
    // and mutate that, instead
    let val = microstatements[microstatements.length - 1];
    if (val.statementType === StatementType.REREF) {
      val = Microstatement.fromVarName(val.alias, scope, microstatements);
    }
    val.statementType = StatementType.LETDEC;
    microstatements.push(
      new Microstatement(
        StatementType.REREF,
        scope,
        true,
        val.outputName,
        val.outputType,
        [],
        [],
        letAlias,
      ),
    );
  }

  static fromConstdeclarationAst(
    constdeclarationAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const constName = '_' + uuid().replace(/-/g, '_');
    const constAlias = constdeclarationAst.get('variable').t;
    const constTypeHint = constdeclarationAst.get('typedec').has()
      ? constdeclarationAst.get('typedec').get('fulltypename').t
      : '';
    const type = scope.deepGet(constTypeHint);
    if (type === null && constTypeHint !== '') {
      // Try to define it if it's a generic type
      const constTypeAst = constdeclarationAst
        .get('typedec')
        .get('fulltypename');
      if (constTypeAst.has('opttypegenerics')) {
        const outerType = scope.deepGet(constTypeAst.get('typename').t) as Type;
        if (outerType === null) {
          throw new Error(`${constTypeAst.get('typename').t}  is not defined
${constdeclarationAst.t} on line ${constdeclarationAst.line}:${
            constdeclarationAst.char
          }`);
        }
        const generics = [];
        const genericAst = constTypeAst.get('opttypegenerics').get('generics');
        generics.push(genericAst.get('fulltypename').t);
        genericAst
          .get('cdr')
          .getAll()
          .forEach((r) => {
            generics.push(r.get('fulltypename').t);
          });
        outerType.solidify(generics, scope);
      }
    }
    Microstatement.fromAssignablesAst(
      constdeclarationAst.get('assignables'),
      scope,
      microstatements,
    );
    // By definition the last microstatement is the const assignment we care about, so we can just
    // mutate its object to rename the output variable name to the name we need instead.
    microstatements.push(
      new Microstatement(
        StatementType.REREF,
        scope,
        true,
        microstatements[microstatements.length - 1].outputName,
        microstatements[microstatements.length - 1].outputType,
        [],
        [],
        constAlias,
      ),
    );
  }

  // DFS recursive algo to get the microstatements in a valid ordering
  static fromStatementsAst(
    statementAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    if (statementAst.has('declarations')) {
      if (statementAst.get('declarations').has('constdeclaration')) {
        Microstatement.fromConstdeclarationAst(
          statementAst.get('declarations').get('constdeclaration'),
          scope,
          microstatements,
        );
      } else {
        Microstatement.fromLetdeclarationAst(
          statementAst.get('declarations').get('letdeclaration'),
          scope,
          microstatements,
        );
      }
    }
    if (statementAst.has('assignments')) {
      Microstatement.fromAssignmentsAst(
        statementAst.get('assignments'),
        scope,
        microstatements,
      );
    }
    if (statementAst.has('assignables')) {
      Microstatement.fromAssignablesAst(
        statementAst.get('assignables').get('assignables'),
        scope,
        microstatements,
      );
    }
    if (statementAst.has('exits')) {
      Microstatement.fromExitsAst(
        statementAst.get('exits'),
        scope,
        microstatements,
      );
    }
    if (statementAst.has('emits')) {
      Microstatement.fromEmitsAst(
        statementAst.get('emits'),
        scope,
        microstatements,
      );
    }

    return microstatements;
  }

  static fromBaseAssignableAst(
    baseAssignableAsts: LPNode[],
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // The base assignables array are a lightly annotated set of primitives that can be combined
    // together to produce an assignable value. Certain combinations of these primitives are invalid
    // and TODO provide good error messaging when these are encountered. A state machine of valid
    // transitions is defined below:
    //
    // null -> { var, obj, fn, const, group }
    // var -> { dot, arraccess, call, eos }
    // obj -> { dot, arraccess, eos }
    // fn -> { call, eos }
    // const -> { dot, eos }
    // group -> { dot, arraccess, eos }
    // call -> { call, arraccess, dot, eos }
    // arraccess -> { arraccess, dot, call, eos }
    //
    // Where `null` is the initial state and `eos` is end-of-statement terminating state. `var` is
    // some variable-name-like value (could be a scope, variable, property, or function name). `obj`
    // is object literal syntax, `fn` is function literal syntax, `const` is a constant literal.
    // `group)` is re-using the function call syntax to handle operator grouping (eg `2 * (3 + 4)`).
    // Because of how operators are mixed in with the assignables, the only time this syntax is used
    // as an operator grouping syntax is if it is the first element in the array. Otherwise it is
    // being used as a function call for a given function (either defined by a variable, an
    // inline-defined function, or a returned function from another call or array access) as `call`.
    // Finally `arraccess` is when an array (and ideally later a HashMap) is accessed. This mode is
    // also abusing the `obj` syntax, but only when it's an array literal with only one value and no
    // `new Array<foo>` type definition *and* when there are prior elements in the list. This means
    // `[0][0]` is unambiguous and would return a Result-wrapped zero value, for instance.
    //
    // The exact meaning of `var.var...` chains varies based on the elements of the array both
    // before and after such a chain. If the start of such a list, and if a `call` is at the end, it
    // could be something like `scope.variable.property.functionName(args)` where `.property` can
    // repeat multiple times over. Basically, to properly parse any `.var` requires both the prior
    // state *and* look-ahead to the next element in the list.
    //
    // All of this to re-iterate that for the sake of compile time, some of the complexities of the
    // grammar have been moved from the LP definition into the compiler itself for performance
    // reasons, explaining the complicated iterative logic that follows.

    let currVal: any = null;
    for (let i = 0; i < baseAssignableAsts.length; i++) {
      const baseassignable = baseAssignableAsts[i].get('baseassignable');
      if (baseassignable.has('methodsep')) {
        if (i === 0) {
          throw new Error(`Invalid start of assignable statement. Cannot begin with a dot (.)
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
        }
        const prevassignable = baseAssignableAsts[i - 1].get('baseassignable');
        if (prevassignable.has('methodsep')) {
          throw new Error(`Invalid property access. You accidentally typed a dot twice in a row.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
        } else if (prevassignable.has('functions')) {
          throw new Error(`Invalid property access. Functions do not have properties.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
        }
        // TODO: Do we even do anything else in this branch?
      } else if (baseassignable.has('variable')) {
        const nextassignable = baseAssignableAsts[i + 1]
          ? baseAssignableAsts[i + 1].get('baseassignable')
          : undefined;
        if (!!nextassignable && nextassignable.has('fncall')) {
          // This is a function call path
          const fncall = nextassignable.get('fncall');
          const argAsts = [];
          if (fncall.get('assignablelist').has()) {
            argAsts.push(fncall.get('assignablelist').get('assignables'));
            fncall
              .get('assignablelist')
              .get('cdr')
              .getAll()
              .forEach((r) => {
                argAsts.push(r.get('assignables'));
              });
          }
          const argMicrostatements = argAsts.map((arg) => {
            Microstatement.fromAssignablesAst(arg, scope, microstatements);
            return microstatements[microstatements.length - 1];
          });
          if (currVal === null) {
            // This is a basic function call
            const realArgNames = argMicrostatements.map(
              (arg) => arg.outputName,
            );
            const realArgTypes = argMicrostatements.map(
              (arg) => arg.outputType,
            );
            // Do a scan of the microstatements for an inner defined closure that might exist.
            const fn = scope.deepGet(
              baseassignable.get('variable').t,
            ) as Array<Fn>;
            if (
              !fn ||
              !(
                fn instanceof Array &&
                fn[0].microstatementInlining instanceof Function
              )
            ) {
              const fnName = baseassignable.get('variable').t;
              let actualFnName: string;
              let inlinedClosure = false;
              for (let i = microstatements.length - 1; i >= 0; i--) {
                if (microstatements[i].alias === fnName) {
                  actualFnName = microstatements[i].outputName;
                  continue;
                }
                if (
                  microstatements[i].outputName === actualFnName &&
                  microstatements[i].statementType === StatementType.CLOSUREDEF
                ) {
                  const m = [
                    ...microstatements,
                    ...microstatements[i].closureStatements,
                  ];
                  const fn = UserFunction.dispatchFn(
                    microstatements[i].fns,
                    realArgTypes,
                    scope,
                  );
                  const interfaceMap = new Map();
                  Object.values(fn.getArguments()).forEach((t: Type, i) =>
                    t.typeApplies(realArgTypes[i], scope, interfaceMap),
                  );
                  Microstatement.closureFromUserFunction(
                    fn,
                    fn.scope || scope,
                    m,
                    interfaceMap,
                  );
                  const closure = m.pop();
                  microstatements.push(
                    ...closure.closureStatements.filter(
                      (s) => s.statementType !== StatementType.EXIT,
                    ),
                  );
                  currVal = microstatements[microstatements.length - 1];
                  inlinedClosure = true;
                  break;
                }
              }
              if (!inlinedClosure) {
                throw new Error(`${
                  baseassignable.get('variable').t
                } is not a function but used as one.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
              }
            } else {
              // Generate the relevant microstatements for this function. UserFunctions get inlined
              // with the return statement turned into a const assignment as the last statement,
              // while built-in functions are kept as function calls with the correct renaming.
              UserFunction.dispatchFn(
                fn,
                realArgTypes,
                scope,
              ).microstatementInlining(realArgNames, scope, microstatements);
              currVal = microstatements[microstatements.length - 1];
            }
          } else if (currVal instanceof Scope) {
            // This is calling a function by its parent scope
            const realArgNames = argMicrostatements.map(
              (arg) => arg.outputName,
            );
            const realArgTypes = argMicrostatements.map(
              (arg) => arg.outputType,
            );
            const fn = currVal.deepGet(
              baseassignable.get('variable').t,
            ) as Array<Fn>;
            if (
              !fn ||
              !(
                fn instanceof Array &&
                fn[0].microstatementInlining instanceof Function
              )
            ) {
              throw new Error(`${
                baseassignable.get('variable').t
              } is not a function but used as one.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
            }
            // Generate the relevant microstatements for this function. UserFunctions get inlined
            // with the return statement turned into a const assignment as the last statement,
            // while built-in functions are kept as function calls with the correct renaming.
            UserFunction.dispatchFn(
              fn,
              realArgTypes,
              scope,
            ).microstatementInlining(realArgNames, scope, microstatements);
            currVal = microstatements[microstatements.length - 1];
          } else {
            // It's a method-style function call
            const realArgNames = [
              currVal.outputName,
              ...argMicrostatements.map((arg) => arg.outputName),
            ];
            const realArgTypes = [
              currVal.outputType,
              ...argMicrostatements.map((arg) => arg.outputType),
            ];
            const fn = scope.deepGet(
              baseassignable.get('variable').t,
            ) as Array<Fn>;
            if (
              !fn ||
              !(
                fn instanceof Array &&
                fn[0].microstatementInlining instanceof Function
              )
            ) {
              throw new Error(`${
                baseassignable.get('variable').t
              } is not a function but used as one.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
            }
            // Generate the relevant microstatements for this function. UserFunctions get inlined
            // with the return statement turned into a const assignment as the last statement,
            // while built-in functions are kept as function calls with the correct renaming.
            UserFunction.dispatchFn(
              fn,
              realArgTypes,
              scope,
            ).microstatementInlining(realArgNames, scope, microstatements);
            currVal = microstatements[microstatements.length - 1];
          }
          // Intentionally skip over the `fncall` block on the next iteration
          i++;
        } else {
          if (currVal === null) {
            let thing = Microstatement.fromVarName(
              baseassignable.get('variable').t,
              scope,
              microstatements,
            );
            if (!thing) {
              thing = scope.deepGet(baseassignable.get('variable').t);
            }
            if (!thing) {
              throw new Error(`${baseassignable.get('variable').t} not found.
  ${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
            }
            currVal = thing;
          } else if (currVal instanceof Scope) {
            const thing = currVal.deepGet(baseassignable.get('variable').t);
            if (!thing) {
              throw new Error(`${
                baseassignable.get('variable').t
              } not found in other scope.
  ${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
            }
            currVal = thing;
          } else if (currVal instanceof Microstatement) {
            const fieldName = baseassignable.get('variable').t;
            const fields = Object.keys(currVal.outputType.properties);
            const fieldNum = fields.indexOf(fieldName);
            if (fieldNum < 0) {
              // Invalid object access
              throw new Error(`${fieldName} property not found.
  ${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
            }
            // Create a new variable to hold the address within the array literal
            const addrName = '_' + uuid().replace(/-/g, '_');
            microstatements.push(
              new Microstatement(
                StatementType.CONSTDEC,
                scope,
                true,
                addrName,
                Type.builtinTypes['int64'],
                [`${fieldNum}`],
                [],
              ),
            );
            // Insert a `register` opcode.
            const opcodes = require('./opcodes').default;
            opcodes.exportScope
              .get('register')[0]
              .microstatementInlining(
                [currVal.outputName, addrName],
                scope,
                microstatements,
              );
            // We'll need a reference to this for later
            const typeRecord = currVal;
            // Set the original to this newly-generated microstatement
            currVal = microstatements[microstatements.length - 1];
            // Now we do something odd, but correct here; we need to replace the `outputType` from
            // `any` to the type that was actually copied so function resolution continues to work
            currVal.outputType = typeRecord.outputType.properties[fieldName];
          } else {
            // What is this?
            throw new Error(`Impossible path found. Bug in compiler, please report!
Previous value type: ${typeof currVal}
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
          }
        }
      } else if (baseassignable.has('constants')) {
        if (currVal !== null) {
          throw new Error(`Unexpected constant value detected.
Previous value type: ${typeof currVal}
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
        }
        Microstatement.fromConstantsAst(
          baseassignable.get('constants'),
          scope,
          microstatements,
        );
        currVal = microstatements[microstatements.length - 1];
      } else if (baseassignable.has('functions')) {
        if (currVal !== null) {
          throw new Error(`Unexpected function definition detected.
Previous value type: ${typeof currVal}
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
        }
        // So the closures eval correctly, we add the alias microstatements to the scope
        // TODO: Is this the right approach?
        microstatements
          .filter((m) => !!m.alias)
          .forEach((m) => scope.put(m.alias, m));
        const fn = UserFunction.fromFunctionsAst(
          baseassignable.get('functions'),
          scope,
        );
        currVal = fn;
      } else if (baseassignable.has('objectliterals')) {
        if (currVal === null) {
          // Has to be a "normal" object literal in this case
          Microstatement.fromObjectLiteralsAst(
            baseassignable.get('objectliterals'),
            scope,
            microstatements,
          );
          currVal = microstatements[microstatements.length - 1];
        } else {
          // Can only be an array accessor syntax
          const objlit = baseassignable.get('objectliterals');
          if (
            objlit.has('typeliteral') ||
            objlit.get('arrayliteral').has('fullarrayliteral')
          ) {
            throw new Error(`Unexpected object literal definition detected.
Previous value type: ${typeof currVal}
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
          }
          const arrbase = objlit.get('arrayliteral').get('arraybase');
          if (
            !arrbase.get('assignablelist').has() ||
            arrbase.get('assignablelist').get('cdr').getAll().length > 0
          ) {
            throw new Error(`Array access must provide only one index value to query the array with
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
          }
          const assignableAst = arrbase
            .get('assignablelist')
            .get('assignables');
          Microstatement.fromAssignablesAst(
            assignableAst,
            scope,
            microstatements,
          );
          const arrIndex = microstatements[microstatements.length - 1];
          if (
            !(currVal instanceof Microstatement) ||
            currVal.outputType.originalType.typename !== 'Array'
          ) {
            throw new Error(`Array access may only be performed on arrays.
Previous value type: ${currVal.outputType.typename}
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
          }
          if (arrIndex.outputType.typename === 'int64') {
            const opcodes = require('./opcodes').default;
            // Create a new variable to hold the `okR` size value
            const sizeName = '_' + uuid().replace(/-/g, '_');
            microstatements.push(
              new Microstatement(
                StatementType.CONSTDEC,
                scope,
                true,
                sizeName,
                Type.builtinTypes['int64'],
                ['8'],
                [],
              ),
            );
            // Insert an `okR` opcode.
            opcodes.exportScope
              .get('okR')[0]
              .microstatementInlining(
                [arrIndex.outputName, sizeName],
                scope,
                microstatements,
              );
            const wrapped = microstatements[microstatements.length - 1];
            // Insert a `resfrom` opcode.
            opcodes.exportScope
              .get('resfrom')[0]
              .microstatementInlining(
                [currVal.outputName, wrapped.outputName],
                scope,
                microstatements,
              );
          } else if (arrIndex.outputType.typename === 'Result<int64>') {
            const opcodes = require('./opcodes').default;
            // Insert a `resfrom` opcode.
            opcodes.exportScope
              .get('resfrom')[0]
              .microstatementInlining(
                [currVal.outputName, arrIndex.outputName],
                scope,
                microstatements,
              );
          } else {
            throw new Error(`Array access must be done with an int64 or Result<int64> value
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
          }
          // We'll need a reference to this for later
          const arrayRecord = currVal;
          // Update to this newly-generated microstatement
          currVal = microstatements[microstatements.length - 1];
          // Now we do something odd, but correct here; we need to replace the `outputType` from
          // `any` to the type that was actually copied so function resolution continues to work
          currVal.outputType = Type.builtinTypes.Result.solidify(
            [Object.values(arrayRecord.outputType.properties)[0].typename],
            scope,
          );
        }
      } else if (baseassignable.has('fncall')) {
        // It's a `fncall` syntax block but it wasn't caught in a function call before, so it's
        // either a function call on a returned function type, or it's an assignable group
        if (!currVal) {
          // It's probably an assignable group
          if (
            !baseassignable.get('fncall').get('assignablelist').has() ||
            baseassignable
              .get('fncall')
              .get('assignablelist')
              .get('cdr')
              .getAll().length > 0
          ) {
            throw new Error(`Expected a group of assignable values, but got a function call signature.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
          }
          // It *is* an assignable group!
          Microstatement.fromAssignablesAst(
            baseassignable
              .get('fncall')
              .get('assignablelist')
              .get('assignables'),
            scope,
            microstatements,
          );
          currVal = microstatements[microstatements.length - 1];
        } else {
          // TODO: handle functions/closures being called from access out of other function returns
          // and the like
        }
      } else {
        throw new Error(`Compiler error! Completely unhandled input.
${baseassignable.t} on line ${baseassignable.line}:${baseassignable.char}`);
      }
    }
    if (!(currVal instanceof Microstatement)) {
      if (currVal instanceof UserFunction) {
        Microstatement.closureDef(
          [currVal],
          currVal.scope || scope,
          microstatements,
        );
      } else if (
        currVal instanceof Array &&
        currVal[0] instanceof UserFunction
      ) {
        Microstatement.closureDef(
          currVal,
          currVal[0].scope || scope,
          microstatements,
        );
      }
    } else if (currVal.statementType !== StatementType.EMIT) {
      microstatements.push(
        new Microstatement(
          StatementType.REREF,
          scope,
          true,
          currVal.outputName,
          currVal.outputType,
          [],
          [],
          currVal.alias,
        ),
      );
    }
  }

  static fromAssignablesAst(
    assignablesAst: LPNode,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const withoperators = assignablesAst.getAll();
    const withOperatorsList = [];
    for (const operatorOrAssignable of withoperators) {
      if (operatorOrAssignable.get('withoperators').has('operators')) {
        const operator = operatorOrAssignable
          .get('withoperators')
          .get('operators')
          .get(1);
        const op = scope.get(operator.t);
        if (op == null || !(op instanceof Array && op[0] instanceof Operator)) {
          throw new Error('Operator ' + operator.t + ' is not defined');
        }
        withOperatorsList.push(op);
      } else if (
        operatorOrAssignable.get('withoperators').has('baseassignablelist')
      ) {
        Microstatement.fromBaseAssignableAst(
          operatorOrAssignable
            .get('withoperators')
            .get('baseassignablelist')
            .getAll(),
          scope,
          microstatements,
        );
        const last = microstatements[microstatements.length - 1];
        withOperatorsList.push(last);
      }
    }
    // Now to combine these operators and values in the correct order. A compiled language could
    // never do something so inefficient, but I don't care about performance right now, so here's
    // the algorithm: while the list length is greater than 1, perform the two steps:
    // 1. Find the operator with the greatest precedence
    // 2. Apply the underlying function to the values on either side of the operator (or just the
    //    right side if the operator is a prefix operator), then replace the operator with the
    //    returned value in the list and delete the impacted values.
    while (withOperatorsList.length > 1) {
      let maxPrecedence = -1;
      let maxOperatorLoc = -1;
      let maxOperatorListLoc = -1;
      for (let i = 0; i < withOperatorsList.length; i++) {
        if (
          withOperatorsList[i] instanceof Array &&
          withOperatorsList[i][0] instanceof Operator
        ) {
          const ops = withOperatorsList[i];
          let op = null;
          let operatorListLoc = -1;
          let operatorPrecedence = -127;
          if (ops.length == 1) {
            op = ops[0];
            operatorListLoc = 0;
          } else {
            // TODO: We need to identify which particular operator applies in this case.
            // We're just going to short-circuit this process on the first operator that matches
            // but we need to come up with a "best match" behavior (ie, if one argument is an int8
            // it may choose the int64-based operator because it was first and it can cast int8 to
            // int64 and then miss the specialized int8 version of the function).
            let left = null;
            if (i != 0) left = withOperatorsList[i - 1];
            let right = null;
            if (i != withOperatorsList.length - 1)
              right = withOperatorsList[i + 1];
            // Skip over any operator that is followed by another operator as it must be a prefix
            // operator (or a syntax error, but we'll catch that later)
            if (right === null || right instanceof Microstatement) {
              for (let j = 0; j < ops.length; j++) {
                if (
                  ops[j].precedence > operatorPrecedence &&
                  ops[j].applicableFunction(
                    !left // Left is special, if two operators are in a row, this one
                      ? null // needs to be a prefix operator for this to work at all
                      : left instanceof Microstatement
                      ? left.outputType
                      : null,
                    right === null ? null : right.outputType,
                    scope,
                  ) != null
                ) {
                  op = ops[j];
                  operatorListLoc = j;
                  operatorPrecedence = op.precedence;
                }
              }
            }
            // During the process of determining the operator ordering, there may be tests that
            // will not match because operator precedence will convert the neighboring types into
            // types that will match. This is complicated and doing this statically will be more
            // difficult, but for now, just skip over these.
            if (op == null) continue;
          }

          if (op.precedence > maxPrecedence) {
            maxPrecedence = op.precedence;
            maxOperatorLoc = i;
            maxOperatorListLoc = operatorListLoc;
          }
        }
      }
      if (maxPrecedence == -1 || maxOperatorLoc == -1) {
        let errMsg = `Cannot resolve operators with remaining statement
${assignablesAst.t}`;
        const withOperatorsTranslation = [];
        for (let i = 0; i < withOperatorsList.length; i++) {
          const node = withOperatorsList[i];
          if (node instanceof Array && node[0] instanceof Operator) {
            withOperatorsTranslation.push(node[0].name);
          } else {
            withOperatorsTranslation.push('<' + node.outputType.typename + '>');
          }
        }
        errMsg += '\n' + withOperatorsTranslation.join(' ');
        throw new Error(errMsg);
      }
      const op = withOperatorsList[maxOperatorLoc][maxOperatorListLoc];
      const realArgNames = [];
      const realArgTypes = [];
      if (!op.isPrefix) {
        const left = withOperatorsList[maxOperatorLoc - 1];
        realArgNames.push(left.outputName);
        realArgTypes.push(left.outputType);
      }
      const right = withOperatorsList[maxOperatorLoc + 1];
      realArgNames.push(right.outputName);
      realArgTypes.push(right.outputType);
      UserFunction.dispatchFn(
        op.potentialFunctions,
        realArgTypes,
        scope,
      ).microstatementInlining(realArgNames, scope, microstatements);
      const last = microstatements[microstatements.length - 1];
      withOperatorsList[maxOperatorLoc] = last;
      withOperatorsList.splice(maxOperatorLoc + 1, 1);
      if (!op.isPrefix) {
        withOperatorsList.splice(maxOperatorLoc - 1, 1);
      }
    }
  }

  static fromStatement(
    statement: Statement,
    microstatements: Array<Microstatement>,
    secondaryScope: Scope | null = null,
  ) {
    let actualStatement = statement;
    if (secondaryScope !== null) {
      const newScope = new Scope(statement.scope);
      newScope.secondaryPar = secondaryScope;
      actualStatement = new Statement(
        statement.statementAst,
        newScope,
        statement.pure,
      );
    }
    Microstatement.fromStatementsAst(
      actualStatement.statementAst,
      actualStatement.scope,
      microstatements,
    );
  }
}

export default Microstatement;
