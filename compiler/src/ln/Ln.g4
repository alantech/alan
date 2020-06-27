grammar Ln;

// Parser rules

module : (blank* imports* (types | constdeclaration | functions | operatormapping | events | handlers | interfaces | exports | blank+)+) | EOF;

blank : (WS | NEWLINE);

imports : (standardImport | fromImport);

standardImport : IMPORT WS dependency (WS AS WS VARNAME)? NEWLINE+;

fromImport: FROM WS dependency WS IMPORT WS varlist NEWLINE+;

dependency : localdependency | globaldependency;

localdependency : (CURDIR (VARNAME | DIRSEP)+) | (PARDIR (VARNAME | DIRSEP)+);

globaldependency : GLOBAL (VARNAME | DIRSEP)+;

types : TYPE blank+ typename blank* typegenerics? blank+ (typebody | EQUALS blank* othertype (blank* OR blank* othertype)*);

othertype : typename blank* typegenerics?;

typename : varn;

typegenerics : OPENGENERIC blank* fulltypename blank* (SEP blank* fulltypename blank*)* CLOSEGENERIC;

fulltypename : varn blank* typegenerics?;

typebody: OPENBODY blank* (WS* typeline)+ blank? CLOSEBODY;

typeline: VARNAME (WS | NEWLINE)? TYPESEP (WS | NEWLINE)? varn NEWLINE*;

functions : FN blank+ ((VARNAME blank*)? OPENARGS arglist? CLOSEARGS blank* ((WS | NEWLINE)? TYPESEP (WS | NEWLINE)? argtype blank*)?)? fullfunctionbody;

fullfunctionbody : functionbody | (EQUALS blank* assignables);

functionbody : OPENBODY blank* statements+ blank* CLOSEBODY;

statements : (declarations | assignments | calls | exits | emits | conditionals) blank+;

declarations : (constdeclaration | letdeclaration);

constdeclaration : CONST blank* (VARNAME blank* TYPESEP (WS | NEWLINE)?)? blank* assignments;

letdeclaration : LET blank* (VARNAME blank* TYPESEP (WS | NEWLINE)?)? blank* assignments;

assignments : varn blank* ((typegenerics? blank* EQUALS blank* assignables) | typegenerics?);

assignables : basicassignables | withoperators;

basicassignables: functions | calls | varn | constants | groups | typeofn | objectliterals;

operatororassignable : operators | basicassignables;

withoperators : (operatororassignable WS*)+;

groups : OPENARGS WS* withoperators WS* CLOSEARGS;

typeofn : TYPE WS* basicassignables;

objectliterals : NEW WS* othertype WS* (arrayliteral | typeliteral | mapliteral);

arrayliteral : OPENARRAY blank* assignablelist blank* CLOSEARRAY;

typeliteral : OPENBODY blank* (assignments blank+)+ CLOSEBODY;

mapliteral : OPENBODY blank* (mapline blank+)* CLOSEBODY;

mapline : assignables WS* TYPESEP WS* assignables;

assignablelist : blank* assignables (SEP blank* assignables)* blank*;

fncall : OPENARGS assignablelist? CLOSEARGS;

calls: (varn WS* fncall (METHODSEP varn WS* fncall)*) | ((constants | OPENARGS assignables CLOSEARGS) (METHODSEP varn WS* fncall)+);

exits : RETURN (blank* assignables)?;

emits : EMIT blank* varn (blank* assignables)?;

conditionals : IF blank* withoperators blank* blocklikes (blank* ELSE blank* (conditionals | blocklikes))?;

blocklikes : functions | functionbody | varn;

constants : (NUMBERCONSTANT | STRINGCONSTANT | BOOLCONSTANT);

operators : (GENERALOPERATORS | TYPESEP | OPENGENERIC | OR | (CLOSEGENERIC+ ((EQUALS+ GENERALOPERATORS*) | (GENERALOPERATORS+))?) | GLOBAL | DIRSEP);

operatormapping : (PREFIX | INFIX) WS ((fntoop WS opprecedence) | (opprecedence WS fntoop));

fntoop : varn WS AS WS operators;

opprecedence : PRECEDENCE WS NUMBERCONSTANT;

events : EVENT blank VARNAME blank* TYPESEP (WS | NEWLINE)? varn;

handlers : ON blank+ eventref blank+ (functions | varn | functionbody);

eventref : varn | calls;

interfaces : INTERFACE WS* VARNAME WS* OPENBODY blank* (interfaceline blank+)* CLOSEBODY;

interfaceline : functiontypeline | operatortypeline | propertytypeline;

functiontypeline : (VARNAME | FN) WS* functiontype;

functiontype : OPENARGS blank* varn blank* (SEP blank* varn blank*)* CLOSEARGS (WS | NEWLINE)? TYPESEP blank* varn;

operatortypeline : (leftarg blank*)? operators blank* rightarg blank* TYPESEP blank* varn;

leftarg : varn;

rightarg : varn;

propertytypeline : VARNAME WS* TYPESEP WS* varn;

argtype : othertype (blank* OR blank* othertype)*;

arglist : VARNAME (WS | NEWLINE)? TYPESEP (WS | NEWLINE)? argtype (SEP VARNAME (WS | NEWLINE)? TYPESEP (WS | NEWLINE)? argtype)*;

exports : EXPORT (WS | NEWLINE)+ (varn | types | constdeclaration | functions | operatormapping | events | interfaces);

varlist : renameablevar (SEP renameablevar)*;

renameablevar : varop (WS AS WS varop)?;

varop : VARNAME | operators;

varn : varsegment+;

varsegment : VARNAME | METHODSEP | arrayaccess;

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

OR : '|';

TYPESEP : ':';

GENERALOPERATORS : [+\-/*^.~`!@#$%&|:;<?=][+\-/*^.~`!@#$%&|:;<>?=]*;

// Next ignored bits of various kinds

NEWLINE : ('\r' | '\n' | '\r\n');

WS : (' ' | '\t')+;

SINGLELINECOMMENT : '//' ~[\r\n]+ -> skip;

MULTILINECOMMENT : '/*' (('*' ~'/') | ~'*')* '*/' -> skip;

// Finally the super-greedy variable-name-like bits

STRINGCONSTANT : ('"' ~["]* '"') | ('\'' ~[']* '\'');

NUMBERCONSTANT : ('0x' [0-9a-fA-F]+) | ([0-9]+ ([.][0-9]+)?);

VARNAME : [a-zA-Z_]+ ([a-zA-Z0-9_])*;

