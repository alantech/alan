grammar Amm;

// Parser rules

module : (blank* (constdeclaration | blank+)* (events | blank+)* (handlers | blank+)+) | EOF;

blank : (WS | NEWLINE);

typename : VARNAME;

typegenerics : OPENGENERIC blank* fulltypename blank* (SEP blank* fulltypename blank*)* CLOSEGENERIC;

fulltypename : typename blank* typegenerics? | VOID;

functions : FN blank+ OPENARGS (VARNAME TYPESEP fulltypename)? CLOSEARGS blank* TYPESEP blank* VOID blank* functionbody;

functionbody : OPENBODY blank* statements+ blank* CLOSEBODY;

statements : (declarations | assignments | calls | emits) blank+;

declarations : (constdeclaration | letdeclaration);

decname : VARNAME;

constdeclaration : CONST blank* decname blank* TYPESEP blank* fulltypename blank* EQUALS blank* assignables;

letdeclaration : LET blank* decname blank* TYPESEP blank* fulltypename blank* EQUALS blank* assignables;

assignments : decname blank* EQUALS blank* assignables;

assignables : functions | calls | constants | VARNAME;

calllist : blank* VARNAME (SEP blank* VARNAME)* blank*;

calls : VARNAME WS* OPENARGS calllist? CLOSEARGS;

emits : EMIT blank* VARNAME (blank* VARNAME)?;

constants : (NUMBERCONSTANT | STRINGCONSTANT | BOOLCONSTANT);

events : EVENT blank VARNAME blank* TYPESEP (typename | VOID);

handlers : ON blank+ VARNAME blank+ functions;

// Lexer rules

// First, keywords

FN : 'fn';

EVENT : 'event';

ON: 'on';

CONST : 'const';

LET : 'let';

EMIT : 'emit';

BOOLCONSTANT : ('true' | 'false');

// Next, sigils in the language

SEP : ',' WS*;

OPENBODY : '{';

CLOSEBODY: '}';

OPENARGS : '(';

CLOSEARGS : ')';

OPENGENERIC : '<';

CLOSEGENERIC : '>';

EQUALS : '=';

VOID : 'void';

TYPESEP : (WS | NEWLINE)? ':' (WS | NEWLINE)?;

// Next ignored bits of various kinds

NEWLINE : ('\r' | '\n' | '\r\n');

WS : (' ' | '\t')+;

// Finally the super-greedy variable-name-like bits

STRINGCONSTANT : ('"' ~["]* '"') | ('\'' ~[']* '\'');

NUMBERCONSTANT : ('0x' [0-9a-fA-F]+) | ([0-9]+ ([.][0-9]+)?);

VARNAME : [a-zA-Z_]+ ([a-zA-Z0-9_])*;

