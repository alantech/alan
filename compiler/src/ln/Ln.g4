grammar Ln;

// Parser rules

module : (blank* imports* (types | (constdeclaration EOS) | functions | operatormapping | events | handlers | interfaces | exports | blank+)+) | EOF;

blank : (WS | NEWLINE);

imports : (standardImport | fromImport);

standardImport : IMPORT WS dependency (WS AS WS VARNAME)? NEWLINE blank*;

fromImport: FROM WS dependency WS IMPORT WS varlist NEWLINE blank*;

dependency : localdependency | globaldependency;

localdependency : (CURDIR (VARNAME | DIRSEP)+) | (PARDIR (VARNAME | DIRSEP)+);

globaldependency : GLOBAL (VARNAME | DIRSEP)+;

types : TYPE blank+ typename (blank* typegenerics)? blank+ (typebody | EQUALS blank* fulltypename);

typename : VARNAME (METHODSEP VARNAME)?;

typegenerics : OPENGENERIC blank* fulltypename blank* (SEP blank* fulltypename blank*)* CLOSEGENERIC;

fulltypename : typename (blank* typegenerics)?;

typebody: OPENBODY blank* typelist blank* CLOSEBODY;

typeline : VARNAME blank* TYPESEP blank* fulltypename;

typelist : typeline blank* (SEP blank* typeline blank*)* SEP?;

arglist : VARNAME blank* TYPESEP blank* fulltypename (SEP VARNAME blank* TYPESEP blank* fulltypename)*;

functions : FN blank+ ((VARNAME blank*)? OPENARGS arglist? CLOSEARGS blank* (blank? TYPESEP blank? fulltypename blank*)?)? fullfunctionbody EOS?;

fullfunctionbody : functionbody | (EQUALS blank* assignables);

functionbody : OPENBODY statements+ blank* CLOSEBODY;

statements : blank* (declarations | exits | emits | assignments | (assignables EOS) | conditionals);

declarations : constdeclaration | letdeclaration;

constdeclaration : CONST blank* VARNAME blank* (TYPESEP blank? fulltypename)? blank* EQUALS blank* assignables EOS;

letdeclaration : LET blank* VARNAME blank* (TYPESEP blank? fulltypename)? blank* EQUALS blank* assignables EOS;

assignments : varn blank* EQUALS blank* assignables EOS;

baseassignable : METHODSEP | VARNAME | constants | functions | fncall | objectliterals;

withoperators : (baseassignable blank*)+ | operators;

assignables : withoperators (blank* withoperators)*;

objectliterals : arrayliteral | typeliteral;

assignablelist : assignables blank* (SEP blank* assignables blank*)* SEP?;

typeassignlist: VARNAME blank* TYPESEP blank* assignables blank* (SEP blank* VARNAME blank* TYPESEP blank* assignables blank*)* SEP?;

literaldec : NEW WS* fulltypename WS*;

arraybase : OPENARRAY blank* assignablelist? blank* CLOSEARRAY;

arrayliteral : arraybase | (literaldec arraybase);

typebase: OPENBODY blank* typeassignlist blank* CLOSEBODY;

typeliteral : literaldec typebase;

fncall : OPENARGS blank* assignablelist? blank* CLOSEARGS;

exits : RETURN (blank* assignables blank*)? EOS;

emits : EMIT blank* eventref (blank* assignables blank*)? EOS;

conditionals : IF blank* assignables blank* blocklikes (blank* ELSE blank* (conditionals | blocklikes))?;

blocklikes : functions | functionbody | eventref;

constants : (NUMBERCONSTANT | STRINGCONSTANT | BOOLCONSTANT);

operators : (GENERALOPERATORS | TYPESEP | OPENGENERIC | (CLOSEGENERIC+ ((EQUALS+ GENERALOPERATORS*) | (GENERALOPERATORS+))?) | GLOBAL | DIRSEP);

operatormapping : (PREFIX | INFIX) WS ((fntoop WS opprecedence) | (opprecedence WS fntoop));

fntoop : eventref WS AS WS operators;

opprecedence : PRECEDENCE WS NUMBERCONSTANT;

events : EVENT blank* VARNAME blank* TYPESEP blank* fulltypename;

eventref : typename;

handlers : ON blank+ eventref blank+ (functions | typename | functionbody);

interfaces : INTERFACE WS* VARNAME WS* (interfacebody | (EQUALS blank* VARNAME));

interfacebody : OPENBODY interfacelist? blank* CLOSEBODY;

interfacelist : blank* interfaceline blank* (SEP blank* interfaceline blank*)* SEP?;

interfaceline : functiontypeline | operatortypeline | propertytypeline;

functiontypeline : (VARNAME | FN) WS* functiontype;

functiontype : OPENARGS blank* fulltypename blank* (SEP blank* fulltypename blank*)* CLOSEARGS blank? TYPESEP blank* fulltypename;

operatortypeline : (leftarg blank*)? operators blank* rightarg blank* TYPESEP blank* fulltypename;

leftarg : fulltypename;

rightarg : fulltypename;

propertytypeline : VARNAME WS* TYPESEP WS* fulltypename;

exports : EXPORT blank+ (eventref | types | constdeclaration | functions | operatormapping | events | interfaces);

varlist : renameablevar (SEP renameablevar)*;

renameablevar : varop (WS+ AS WS+ varop)?;

varop : VARNAME | operators;

varn : varsegment+;

varsegment : VARNAME | (blank* METHODSEP) | arrayaccess;

arrayaccess : OPENARRAY WS* assignables WS* CLOSEARRAY;

// Lexer rules

// First, keywords

IMPORT : 'import';

FROM : 'from';

TYPE : 'type';

FN : 'fn';

EVENT : 'event';

ON: 'on';

EXPORT : 'export';

CONST : 'const';

LET : 'let';

RETURN : 'return';

EMIT : 'emit';

AS : 'as';

BOOLCONSTANT : ('true' | 'false');

PREFIX : 'prefix';

INFIX : 'infix';

PRECEDENCE : 'precedence';

IF : 'if';

ELSE : 'else';

NEW : 'new';

INTERFACE : 'interface';

// Next, sigils in the language

SEP : ',' WS*;

OPENBODY : '{';

CLOSEBODY: '}';

OPENARGS : '(';

CLOSEARGS : ')';

OPENGENERIC : '<';

CLOSEGENERIC : '>';

OPENARRAY : '[';

CLOSEARRAY : ']';

METHODSEP : '.';

EQUALS : '=';

GLOBAL : '@';

CURDIR : './';

PARDIR : '../';

DIRSEP : '/';

TYPESEP : ':';

EOS : ';';

// Next ignored bits of various kinds

NEWLINE : ('\r' | '\n' | '\r\n');

WS : (' ' | '\t')+;

SINGLELINECOMMENT : '//' ~[\r\n]+ -> skip;

MULTILINECOMMENT : '/*' (('*' ~'/') | ~'*')* '*/' -> skip;

// Finally the super-greedy variable-name-like bits

STRINGCONSTANT : ('"' ~["]* '"') | ('\'' ~[']* '\'');

NUMBERCONSTANT : ('0x' [0-9a-fA-F]+) | ([-]? [0-9]+ ([.][0-9]+)?);

GENERALOPERATORS : [+\-/*^.~`!@#$%&|:<?=][+\-/*^.~`!@#$%&|:<>?=]*;

VARNAME : [a-zA-Z_]+ ([a-zA-Z0-9_])*;

