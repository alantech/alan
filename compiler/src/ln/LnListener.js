// Generated from Ln.g4 by ANTLR 4.8
// jshint ignore: start
var antlr4 = require('antlr4/index');

// This class defines a complete listener for a parse tree produced by LnParser.
function LnListener() {
	antlr4.tree.ParseTreeListener.call(this);
	return this;
}

LnListener.prototype = Object.create(antlr4.tree.ParseTreeListener.prototype);
LnListener.prototype.constructor = LnListener;

// Enter a parse tree produced by LnParser#module.
LnListener.prototype.enterModule = function(ctx) {
};

// Exit a parse tree produced by LnParser#module.
LnListener.prototype.exitModule = function(ctx) {
};


// Enter a parse tree produced by LnParser#blank.
LnListener.prototype.enterBlank = function(ctx) {
};

// Exit a parse tree produced by LnParser#blank.
LnListener.prototype.exitBlank = function(ctx) {
};


// Enter a parse tree produced by LnParser#imports.
LnListener.prototype.enterImports = function(ctx) {
};

// Exit a parse tree produced by LnParser#imports.
LnListener.prototype.exitImports = function(ctx) {
};


// Enter a parse tree produced by LnParser#standardImport.
LnListener.prototype.enterStandardImport = function(ctx) {
};

// Exit a parse tree produced by LnParser#standardImport.
LnListener.prototype.exitStandardImport = function(ctx) {
};


// Enter a parse tree produced by LnParser#fromImport.
LnListener.prototype.enterFromImport = function(ctx) {
};

// Exit a parse tree produced by LnParser#fromImport.
LnListener.prototype.exitFromImport = function(ctx) {
};


// Enter a parse tree produced by LnParser#dependency.
LnListener.prototype.enterDependency = function(ctx) {
};

// Exit a parse tree produced by LnParser#dependency.
LnListener.prototype.exitDependency = function(ctx) {
};


// Enter a parse tree produced by LnParser#localdependency.
LnListener.prototype.enterLocaldependency = function(ctx) {
};

// Exit a parse tree produced by LnParser#localdependency.
LnListener.prototype.exitLocaldependency = function(ctx) {
};


// Enter a parse tree produced by LnParser#globaldependency.
LnListener.prototype.enterGlobaldependency = function(ctx) {
};

// Exit a parse tree produced by LnParser#globaldependency.
LnListener.prototype.exitGlobaldependency = function(ctx) {
};


// Enter a parse tree produced by LnParser#types.
LnListener.prototype.enterTypes = function(ctx) {
};

// Exit a parse tree produced by LnParser#types.
LnListener.prototype.exitTypes = function(ctx) {
};


// Enter a parse tree produced by LnParser#othertype.
LnListener.prototype.enterOthertype = function(ctx) {
};

// Exit a parse tree produced by LnParser#othertype.
LnListener.prototype.exitOthertype = function(ctx) {
};


// Enter a parse tree produced by LnParser#typename.
LnListener.prototype.enterTypename = function(ctx) {
};

// Exit a parse tree produced by LnParser#typename.
LnListener.prototype.exitTypename = function(ctx) {
};


// Enter a parse tree produced by LnParser#typegenerics.
LnListener.prototype.enterTypegenerics = function(ctx) {
};

// Exit a parse tree produced by LnParser#typegenerics.
LnListener.prototype.exitTypegenerics = function(ctx) {
};


// Enter a parse tree produced by LnParser#fulltypename.
LnListener.prototype.enterFulltypename = function(ctx) {
};

// Exit a parse tree produced by LnParser#fulltypename.
LnListener.prototype.exitFulltypename = function(ctx) {
};


// Enter a parse tree produced by LnParser#typebody.
LnListener.prototype.enterTypebody = function(ctx) {
};

// Exit a parse tree produced by LnParser#typebody.
LnListener.prototype.exitTypebody = function(ctx) {
};


// Enter a parse tree produced by LnParser#typeline.
LnListener.prototype.enterTypeline = function(ctx) {
};

// Exit a parse tree produced by LnParser#typeline.
LnListener.prototype.exitTypeline = function(ctx) {
};


// Enter a parse tree produced by LnParser#functions.
LnListener.prototype.enterFunctions = function(ctx) {
};

// Exit a parse tree produced by LnParser#functions.
LnListener.prototype.exitFunctions = function(ctx) {
};


// Enter a parse tree produced by LnParser#fullfunctionbody.
LnListener.prototype.enterFullfunctionbody = function(ctx) {
};

// Exit a parse tree produced by LnParser#fullfunctionbody.
LnListener.prototype.exitFullfunctionbody = function(ctx) {
};


// Enter a parse tree produced by LnParser#functionbody.
LnListener.prototype.enterFunctionbody = function(ctx) {
};

// Exit a parse tree produced by LnParser#functionbody.
LnListener.prototype.exitFunctionbody = function(ctx) {
};


// Enter a parse tree produced by LnParser#statements.
LnListener.prototype.enterStatements = function(ctx) {
};

// Exit a parse tree produced by LnParser#statements.
LnListener.prototype.exitStatements = function(ctx) {
};


// Enter a parse tree produced by LnParser#declarations.
LnListener.prototype.enterDeclarations = function(ctx) {
};

// Exit a parse tree produced by LnParser#declarations.
LnListener.prototype.exitDeclarations = function(ctx) {
};


// Enter a parse tree produced by LnParser#constdeclaration.
LnListener.prototype.enterConstdeclaration = function(ctx) {
};

// Exit a parse tree produced by LnParser#constdeclaration.
LnListener.prototype.exitConstdeclaration = function(ctx) {
};


// Enter a parse tree produced by LnParser#letdeclaration.
LnListener.prototype.enterLetdeclaration = function(ctx) {
};

// Exit a parse tree produced by LnParser#letdeclaration.
LnListener.prototype.exitLetdeclaration = function(ctx) {
};


// Enter a parse tree produced by LnParser#assignments.
LnListener.prototype.enterAssignments = function(ctx) {
};

// Exit a parse tree produced by LnParser#assignments.
LnListener.prototype.exitAssignments = function(ctx) {
};


// Enter a parse tree produced by LnParser#assignables.
LnListener.prototype.enterAssignables = function(ctx) {
};

// Exit a parse tree produced by LnParser#assignables.
LnListener.prototype.exitAssignables = function(ctx) {
};


// Enter a parse tree produced by LnParser#basicassignables.
LnListener.prototype.enterBasicassignables = function(ctx) {
};

// Exit a parse tree produced by LnParser#basicassignables.
LnListener.prototype.exitBasicassignables = function(ctx) {
};


// Enter a parse tree produced by LnParser#operatororassignable.
LnListener.prototype.enterOperatororassignable = function(ctx) {
};

// Exit a parse tree produced by LnParser#operatororassignable.
LnListener.prototype.exitOperatororassignable = function(ctx) {
};


// Enter a parse tree produced by LnParser#withoperators.
LnListener.prototype.enterWithoperators = function(ctx) {
};

// Exit a parse tree produced by LnParser#withoperators.
LnListener.prototype.exitWithoperators = function(ctx) {
};


// Enter a parse tree produced by LnParser#groups.
LnListener.prototype.enterGroups = function(ctx) {
};

// Exit a parse tree produced by LnParser#groups.
LnListener.prototype.exitGroups = function(ctx) {
};


// Enter a parse tree produced by LnParser#typeofn.
LnListener.prototype.enterTypeofn = function(ctx) {
};

// Exit a parse tree produced by LnParser#typeofn.
LnListener.prototype.exitTypeofn = function(ctx) {
};


// Enter a parse tree produced by LnParser#objectliterals.
LnListener.prototype.enterObjectliterals = function(ctx) {
};

// Exit a parse tree produced by LnParser#objectliterals.
LnListener.prototype.exitObjectliterals = function(ctx) {
};


// Enter a parse tree produced by LnParser#arrayliteral.
LnListener.prototype.enterArrayliteral = function(ctx) {
};

// Exit a parse tree produced by LnParser#arrayliteral.
LnListener.prototype.exitArrayliteral = function(ctx) {
};


// Enter a parse tree produced by LnParser#typeliteral.
LnListener.prototype.enterTypeliteral = function(ctx) {
};

// Exit a parse tree produced by LnParser#typeliteral.
LnListener.prototype.exitTypeliteral = function(ctx) {
};


// Enter a parse tree produced by LnParser#mapliteral.
LnListener.prototype.enterMapliteral = function(ctx) {
};

// Exit a parse tree produced by LnParser#mapliteral.
LnListener.prototype.exitMapliteral = function(ctx) {
};


// Enter a parse tree produced by LnParser#mapline.
LnListener.prototype.enterMapline = function(ctx) {
};

// Exit a parse tree produced by LnParser#mapline.
LnListener.prototype.exitMapline = function(ctx) {
};


// Enter a parse tree produced by LnParser#assignablelist.
LnListener.prototype.enterAssignablelist = function(ctx) {
};

// Exit a parse tree produced by LnParser#assignablelist.
LnListener.prototype.exitAssignablelist = function(ctx) {
};


// Enter a parse tree produced by LnParser#fncall.
LnListener.prototype.enterFncall = function(ctx) {
};

// Exit a parse tree produced by LnParser#fncall.
LnListener.prototype.exitFncall = function(ctx) {
};


// Enter a parse tree produced by LnParser#calls.
LnListener.prototype.enterCalls = function(ctx) {
};

// Exit a parse tree produced by LnParser#calls.
LnListener.prototype.exitCalls = function(ctx) {
};


// Enter a parse tree produced by LnParser#exits.
LnListener.prototype.enterExits = function(ctx) {
};

// Exit a parse tree produced by LnParser#exits.
LnListener.prototype.exitExits = function(ctx) {
};


// Enter a parse tree produced by LnParser#emits.
LnListener.prototype.enterEmits = function(ctx) {
};

// Exit a parse tree produced by LnParser#emits.
LnListener.prototype.exitEmits = function(ctx) {
};


// Enter a parse tree produced by LnParser#conditionals.
LnListener.prototype.enterConditionals = function(ctx) {
};

// Exit a parse tree produced by LnParser#conditionals.
LnListener.prototype.exitConditionals = function(ctx) {
};


// Enter a parse tree produced by LnParser#blocklikes.
LnListener.prototype.enterBlocklikes = function(ctx) {
};

// Exit a parse tree produced by LnParser#blocklikes.
LnListener.prototype.exitBlocklikes = function(ctx) {
};


// Enter a parse tree produced by LnParser#constants.
LnListener.prototype.enterConstants = function(ctx) {
};

// Exit a parse tree produced by LnParser#constants.
LnListener.prototype.exitConstants = function(ctx) {
};


// Enter a parse tree produced by LnParser#operators.
LnListener.prototype.enterOperators = function(ctx) {
};

// Exit a parse tree produced by LnParser#operators.
LnListener.prototype.exitOperators = function(ctx) {
};


// Enter a parse tree produced by LnParser#operatormapping.
LnListener.prototype.enterOperatormapping = function(ctx) {
};

// Exit a parse tree produced by LnParser#operatormapping.
LnListener.prototype.exitOperatormapping = function(ctx) {
};


// Enter a parse tree produced by LnParser#fntoop.
LnListener.prototype.enterFntoop = function(ctx) {
};

// Exit a parse tree produced by LnParser#fntoop.
LnListener.prototype.exitFntoop = function(ctx) {
};


// Enter a parse tree produced by LnParser#opprecedence.
LnListener.prototype.enterOpprecedence = function(ctx) {
};

// Exit a parse tree produced by LnParser#opprecedence.
LnListener.prototype.exitOpprecedence = function(ctx) {
};


// Enter a parse tree produced by LnParser#events.
LnListener.prototype.enterEvents = function(ctx) {
};

// Exit a parse tree produced by LnParser#events.
LnListener.prototype.exitEvents = function(ctx) {
};


// Enter a parse tree produced by LnParser#handlers.
LnListener.prototype.enterHandlers = function(ctx) {
};

// Exit a parse tree produced by LnParser#handlers.
LnListener.prototype.exitHandlers = function(ctx) {
};


// Enter a parse tree produced by LnParser#eventref.
LnListener.prototype.enterEventref = function(ctx) {
};

// Exit a parse tree produced by LnParser#eventref.
LnListener.prototype.exitEventref = function(ctx) {
};


// Enter a parse tree produced by LnParser#interfaces.
LnListener.prototype.enterInterfaces = function(ctx) {
};

// Exit a parse tree produced by LnParser#interfaces.
LnListener.prototype.exitInterfaces = function(ctx) {
};


// Enter a parse tree produced by LnParser#interfaceline.
LnListener.prototype.enterInterfaceline = function(ctx) {
};

// Exit a parse tree produced by LnParser#interfaceline.
LnListener.prototype.exitInterfaceline = function(ctx) {
};


// Enter a parse tree produced by LnParser#functiontypeline.
LnListener.prototype.enterFunctiontypeline = function(ctx) {
};

// Exit a parse tree produced by LnParser#functiontypeline.
LnListener.prototype.exitFunctiontypeline = function(ctx) {
};


// Enter a parse tree produced by LnParser#functiontype.
LnListener.prototype.enterFunctiontype = function(ctx) {
};

// Exit a parse tree produced by LnParser#functiontype.
LnListener.prototype.exitFunctiontype = function(ctx) {
};


// Enter a parse tree produced by LnParser#operatortypeline.
LnListener.prototype.enterOperatortypeline = function(ctx) {
};

// Exit a parse tree produced by LnParser#operatortypeline.
LnListener.prototype.exitOperatortypeline = function(ctx) {
};


// Enter a parse tree produced by LnParser#leftarg.
LnListener.prototype.enterLeftarg = function(ctx) {
};

// Exit a parse tree produced by LnParser#leftarg.
LnListener.prototype.exitLeftarg = function(ctx) {
};


// Enter a parse tree produced by LnParser#rightarg.
LnListener.prototype.enterRightarg = function(ctx) {
};

// Exit a parse tree produced by LnParser#rightarg.
LnListener.prototype.exitRightarg = function(ctx) {
};


// Enter a parse tree produced by LnParser#propertytypeline.
LnListener.prototype.enterPropertytypeline = function(ctx) {
};

// Exit a parse tree produced by LnParser#propertytypeline.
LnListener.prototype.exitPropertytypeline = function(ctx) {
};


// Enter a parse tree produced by LnParser#argtype.
LnListener.prototype.enterArgtype = function(ctx) {
};

// Exit a parse tree produced by LnParser#argtype.
LnListener.prototype.exitArgtype = function(ctx) {
};


// Enter a parse tree produced by LnParser#arglist.
LnListener.prototype.enterArglist = function(ctx) {
};

// Exit a parse tree produced by LnParser#arglist.
LnListener.prototype.exitArglist = function(ctx) {
};


// Enter a parse tree produced by LnParser#exports.
LnListener.prototype.enterExports = function(ctx) {
};

// Exit a parse tree produced by LnParser#exports.
LnListener.prototype.exitExports = function(ctx) {
};


// Enter a parse tree produced by LnParser#varlist.
LnListener.prototype.enterVarlist = function(ctx) {
};

// Exit a parse tree produced by LnParser#varlist.
LnListener.prototype.exitVarlist = function(ctx) {
};


// Enter a parse tree produced by LnParser#renameablevar.
LnListener.prototype.enterRenameablevar = function(ctx) {
};

// Exit a parse tree produced by LnParser#renameablevar.
LnListener.prototype.exitRenameablevar = function(ctx) {
};


// Enter a parse tree produced by LnParser#varop.
LnListener.prototype.enterVarop = function(ctx) {
};

// Exit a parse tree produced by LnParser#varop.
LnListener.prototype.exitVarop = function(ctx) {
};


// Enter a parse tree produced by LnParser#varn.
LnListener.prototype.enterVarn = function(ctx) {
};

// Exit a parse tree produced by LnParser#varn.
LnListener.prototype.exitVarn = function(ctx) {
};


// Enter a parse tree produced by LnParser#varsegment.
LnListener.prototype.enterVarsegment = function(ctx) {
};

// Exit a parse tree produced by LnParser#varsegment.
LnListener.prototype.exitVarsegment = function(ctx) {
};


// Enter a parse tree produced by LnParser#arrayaccess.
LnListener.prototype.enterArrayaccess = function(ctx) {
};

// Exit a parse tree produced by LnParser#arrayaccess.
LnListener.prototype.exitArrayaccess = function(ctx) {
};



exports.LnListener = LnListener;