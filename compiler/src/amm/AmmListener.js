// Generated from Amm.g4 by ANTLR 4.7.2
// jshint ignore: start
var antlr4 = require('antlr4/index');

// This class defines a complete listener for a parse tree produced by AmmParser.
function AmmListener() {
	antlr4.tree.ParseTreeListener.call(this);
	return this;
}

AmmListener.prototype = Object.create(antlr4.tree.ParseTreeListener.prototype);
AmmListener.prototype.constructor = AmmListener;

// Enter a parse tree produced by AmmParser#module.
AmmListener.prototype.enterModule = function(ctx) {
};

// Exit a parse tree produced by AmmParser#module.
AmmListener.prototype.exitModule = function(ctx) {
};


// Enter a parse tree produced by AmmParser#blank.
AmmListener.prototype.enterBlank = function(ctx) {
};

// Exit a parse tree produced by AmmParser#blank.
AmmListener.prototype.exitBlank = function(ctx) {
};


// Enter a parse tree produced by AmmParser#types.
AmmListener.prototype.enterTypes = function(ctx) {
};

// Exit a parse tree produced by AmmParser#types.
AmmListener.prototype.exitTypes = function(ctx) {
};


// Enter a parse tree produced by AmmParser#othertype.
AmmListener.prototype.enterOthertype = function(ctx) {
};

// Exit a parse tree produced by AmmParser#othertype.
AmmListener.prototype.exitOthertype = function(ctx) {
};


// Enter a parse tree produced by AmmParser#typename.
AmmListener.prototype.enterTypename = function(ctx) {
};

// Exit a parse tree produced by AmmParser#typename.
AmmListener.prototype.exitTypename = function(ctx) {
};


// Enter a parse tree produced by AmmParser#typegenerics.
AmmListener.prototype.enterTypegenerics = function(ctx) {
};

// Exit a parse tree produced by AmmParser#typegenerics.
AmmListener.prototype.exitTypegenerics = function(ctx) {
};


// Enter a parse tree produced by AmmParser#fulltypename.
AmmListener.prototype.enterFulltypename = function(ctx) {
};

// Exit a parse tree produced by AmmParser#fulltypename.
AmmListener.prototype.exitFulltypename = function(ctx) {
};


// Enter a parse tree produced by AmmParser#typebody.
AmmListener.prototype.enterTypebody = function(ctx) {
};

// Exit a parse tree produced by AmmParser#typebody.
AmmListener.prototype.exitTypebody = function(ctx) {
};


// Enter a parse tree produced by AmmParser#typeline.
AmmListener.prototype.enterTypeline = function(ctx) {
};

// Exit a parse tree produced by AmmParser#typeline.
AmmListener.prototype.exitTypeline = function(ctx) {
};


// Enter a parse tree produced by AmmParser#functions.
AmmListener.prototype.enterFunctions = function(ctx) {
};

// Exit a parse tree produced by AmmParser#functions.
AmmListener.prototype.exitFunctions = function(ctx) {
};


// Enter a parse tree produced by AmmParser#functionbody.
AmmListener.prototype.enterFunctionbody = function(ctx) {
};

// Exit a parse tree produced by AmmParser#functionbody.
AmmListener.prototype.exitFunctionbody = function(ctx) {
};


// Enter a parse tree produced by AmmParser#statements.
AmmListener.prototype.enterStatements = function(ctx) {
};

// Exit a parse tree produced by AmmParser#statements.
AmmListener.prototype.exitStatements = function(ctx) {
};


// Enter a parse tree produced by AmmParser#declarations.
AmmListener.prototype.enterDeclarations = function(ctx) {
};

// Exit a parse tree produced by AmmParser#declarations.
AmmListener.prototype.exitDeclarations = function(ctx) {
};


// Enter a parse tree produced by AmmParser#decname.
AmmListener.prototype.enterDecname = function(ctx) {
};

// Exit a parse tree produced by AmmParser#decname.
AmmListener.prototype.exitDecname = function(ctx) {
};


// Enter a parse tree produced by AmmParser#constdeclaration.
AmmListener.prototype.enterConstdeclaration = function(ctx) {
};

// Exit a parse tree produced by AmmParser#constdeclaration.
AmmListener.prototype.exitConstdeclaration = function(ctx) {
};


// Enter a parse tree produced by AmmParser#letdeclaration.
AmmListener.prototype.enterLetdeclaration = function(ctx) {
};

// Exit a parse tree produced by AmmParser#letdeclaration.
AmmListener.prototype.exitLetdeclaration = function(ctx) {
};


// Enter a parse tree produced by AmmParser#assignments.
AmmListener.prototype.enterAssignments = function(ctx) {
};

// Exit a parse tree produced by AmmParser#assignments.
AmmListener.prototype.exitAssignments = function(ctx) {
};


// Enter a parse tree produced by AmmParser#assignables.
AmmListener.prototype.enterAssignables = function(ctx) {
};

// Exit a parse tree produced by AmmParser#assignables.
AmmListener.prototype.exitAssignables = function(ctx) {
};


// Enter a parse tree produced by AmmParser#objectliterals.
AmmListener.prototype.enterObjectliterals = function(ctx) {
};

// Exit a parse tree produced by AmmParser#objectliterals.
AmmListener.prototype.exitObjectliterals = function(ctx) {
};


// Enter a parse tree produced by AmmParser#arrayliteral.
AmmListener.prototype.enterArrayliteral = function(ctx) {
};

// Exit a parse tree produced by AmmParser#arrayliteral.
AmmListener.prototype.exitArrayliteral = function(ctx) {
};


// Enter a parse tree produced by AmmParser#typeliteral.
AmmListener.prototype.enterTypeliteral = function(ctx) {
};

// Exit a parse tree produced by AmmParser#typeliteral.
AmmListener.prototype.exitTypeliteral = function(ctx) {
};


// Enter a parse tree produced by AmmParser#mapliteral.
AmmListener.prototype.enterMapliteral = function(ctx) {
};

// Exit a parse tree produced by AmmParser#mapliteral.
AmmListener.prototype.exitMapliteral = function(ctx) {
};


// Enter a parse tree produced by AmmParser#mapline.
AmmListener.prototype.enterMapline = function(ctx) {
};

// Exit a parse tree produced by AmmParser#mapline.
AmmListener.prototype.exitMapline = function(ctx) {
};


// Enter a parse tree produced by AmmParser#assignablelist.
AmmListener.prototype.enterAssignablelist = function(ctx) {
};

// Exit a parse tree produced by AmmParser#assignablelist.
AmmListener.prototype.exitAssignablelist = function(ctx) {
};


// Enter a parse tree produced by AmmParser#calllist.
AmmListener.prototype.enterCalllist = function(ctx) {
};

// Exit a parse tree produced by AmmParser#calllist.
AmmListener.prototype.exitCalllist = function(ctx) {
};


// Enter a parse tree produced by AmmParser#calls.
AmmListener.prototype.enterCalls = function(ctx) {
};

// Exit a parse tree produced by AmmParser#calls.
AmmListener.prototype.exitCalls = function(ctx) {
};


// Enter a parse tree produced by AmmParser#emits.
AmmListener.prototype.enterEmits = function(ctx) {
};

// Exit a parse tree produced by AmmParser#emits.
AmmListener.prototype.exitEmits = function(ctx) {
};


// Enter a parse tree produced by AmmParser#constants.
AmmListener.prototype.enterConstants = function(ctx) {
};

// Exit a parse tree produced by AmmParser#constants.
AmmListener.prototype.exitConstants = function(ctx) {
};


// Enter a parse tree produced by AmmParser#events.
AmmListener.prototype.enterEvents = function(ctx) {
};

// Exit a parse tree produced by AmmParser#events.
AmmListener.prototype.exitEvents = function(ctx) {
};


// Enter a parse tree produced by AmmParser#handlers.
AmmListener.prototype.enterHandlers = function(ctx) {
};

// Exit a parse tree produced by AmmParser#handlers.
AmmListener.prototype.exitHandlers = function(ctx) {
};



exports.AmmListener = AmmListener;