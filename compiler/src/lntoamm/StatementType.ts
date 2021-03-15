enum StatementType {
  CONSTDEC = 'CONSTDEC',
  LETDEC = 'LETDEC',
  ASSIGNMENT = 'ASSIGNMENT',
  CALL = 'CALL',
  EMIT = 'EMIT',
  REREF = 'REREF',
  CLOSURE =  'CLOSURE',
  ARG = 'ARG',
  ENTERFN = 'ENTERFN',
  EXIT = 'EXIT',
  CLOSUREDEF = 'CLOSUREDEF',

  /**
   * This requires some explanation. We need to be able to grab the "tail" of a
   * function - that is, the portion of a function from a certain point until
   * the end - in certain cases (right now, only for handling if/else, but there
   * may be more use cases in the future) and to shove it into an opcode call.
   * This results in a problem: we mark the starting point of the fn tail with
   * the TAIL mStatement, but we have to actually use the tail fn in a mStmt.
   * To get rid of this crud, a TAIL should assume that these actions will be
   * taken when the tail function is generated:
   * - the tail fn will be inserted before the TAIL mStmt
   * - the name of the tail fn will be appended to the mStmt's args
   * - the TAIL mStmt will be turned into an EXIT mStmt (and hence be the last
   *    mStmt in the current fn)
   */
  TAIL = 'TAIL',
  ENTERCONDFN = 'ENTERCONDFN',
}

export default StatementType

