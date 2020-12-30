module.exports = {
  LnLexer: require('./LnLexer').LnLexer,
  LnParser: require('./LnParser').LnParser,
  lp: require('./ln').default,
  stripcomments: require('./ln').stripcomments,
}
