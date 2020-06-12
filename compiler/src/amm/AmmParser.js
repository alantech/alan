// Generated from Amm.g4 by ANTLR 4.8
// jshint ignore: start
var antlr4 = require('antlr4/index');
var AmmListener = require('./AmmListener').AmmListener;
var grammarFileName = "Amm.g4";


var serializedATN = ["\u0003\u608b\ua72a\u8133\ub9ed\u417c\u3be7\u7786\u5964",
    "\u0003\u0018\u0186\u0004\u0002\t\u0002\u0004\u0003\t\u0003\u0004\u0004",
    "\t\u0004\u0004\u0005\t\u0005\u0004\u0006\t\u0006\u0004\u0007\t\u0007",
    "\u0004\b\t\b\u0004\t\t\t\u0004\n\t\n\u0004\u000b\t\u000b\u0004\f\t\f",
    "\u0004\r\t\r\u0004\u000e\t\u000e\u0004\u000f\t\u000f\u0004\u0010\t\u0010",
    "\u0004\u0011\t\u0011\u0004\u0012\t\u0012\u0004\u0013\t\u0013\u0004\u0014",
    "\t\u0014\u0004\u0015\t\u0015\u0003\u0002\u0007\u0002,\n\u0002\f\u0002",
    "\u000e\u0002/\u000b\u0002\u0003\u0002\u0003\u0002\u0006\u00023\n\u0002",
    "\r\u0002\u000e\u00024\u0007\u00027\n\u0002\f\u0002\u000e\u0002:\u000b",
    "\u0002\u0003\u0002\u0003\u0002\u0006\u0002>\n\u0002\r\u0002\u000e\u0002",
    "?\u0007\u0002B\n\u0002\f\u0002\u000e\u0002E\u000b\u0002\u0003\u0002",
    "\u0003\u0002\u0006\u0002I\n\u0002\r\u0002\u000e\u0002J\u0006\u0002M",
    "\n\u0002\r\u0002\u000e\u0002N\u0003\u0002\u0005\u0002R\n\u0002\u0003",
    "\u0003\u0003\u0003\u0003\u0004\u0003\u0004\u0003\u0005\u0003\u0005\u0007",
    "\u0005Z\n\u0005\f\u0005\u000e\u0005]\u000b\u0005\u0003\u0005\u0003\u0005",
    "\u0007\u0005a\n\u0005\f\u0005\u000e\u0005d\u000b\u0005\u0003\u0005\u0003",
    "\u0005\u0007\u0005h\n\u0005\f\u0005\u000e\u0005k\u000b\u0005\u0003\u0005",
    "\u0003\u0005\u0007\u0005o\n\u0005\f\u0005\u000e\u0005r\u000b\u0005\u0007",
    "\u0005t\n\u0005\f\u0005\u000e\u0005w\u000b\u0005\u0003\u0005\u0003\u0005",
    "\u0003\u0006\u0003\u0006\u0007\u0006}\n\u0006\f\u0006\u000e\u0006\u0080",
    "\u000b\u0006\u0003\u0006\u0005\u0006\u0083\n\u0006\u0003\u0006\u0005",
    "\u0006\u0086\n\u0006\u0003\u0007\u0003\u0007\u0006\u0007\u008a\n\u0007",
    "\r\u0007\u000e\u0007\u008b\u0003\u0007\u0003\u0007\u0003\u0007\u0003",
    "\u0007\u0005\u0007\u0092\n\u0007\u0003\u0007\u0003\u0007\u0007\u0007",
    "\u0096\n\u0007\f\u0007\u000e\u0007\u0099\u000b\u0007\u0003\u0007\u0003",
    "\u0007\u0007\u0007\u009d\n\u0007\f\u0007\u000e\u0007\u00a0\u000b\u0007",
    "\u0003\u0007\u0003\u0007\u0007\u0007\u00a4\n\u0007\f\u0007\u000e\u0007",
    "\u00a7\u000b\u0007\u0003\u0007\u0003\u0007\u0003\b\u0003\b\u0007\b\u00ad",
    "\n\b\f\b\u000e\b\u00b0\u000b\b\u0003\b\u0006\b\u00b3\n\b\r\b\u000e\b",
    "\u00b4\u0003\b\u0007\b\u00b8\n\b\f\b\u000e\b\u00bb\u000b\b\u0003\b\u0003",
    "\b\u0003\t\u0003\t\u0003\t\u0003\t\u0005\t\u00c3\n\t\u0003\t\u0006\t",
    "\u00c6\n\t\r\t\u000e\t\u00c7\u0003\n\u0003\n\u0005\n\u00cc\n\n\u0003",
    "\u000b\u0003\u000b\u0003\f\u0003\f\u0007\f\u00d2\n\f\f\f\u000e\f\u00d5",
    "\u000b\f\u0003\f\u0003\f\u0007\f\u00d9\n\f\f\f\u000e\f\u00dc\u000b\f",
    "\u0003\f\u0003\f\u0007\f\u00e0\n\f\f\f\u000e\f\u00e3\u000b\f\u0003\f",
    "\u0003\f\u0007\f\u00e7\n\f\f\f\u000e\f\u00ea\u000b\f\u0003\f\u0003\f",
    "\u0007\f\u00ee\n\f\f\f\u000e\f\u00f1\u000b\f\u0003\f\u0003\f\u0003\r",
    "\u0003\r\u0007\r\u00f7\n\r\f\r\u000e\r\u00fa\u000b\r\u0003\r\u0003\r",
    "\u0007\r\u00fe\n\r\f\r\u000e\r\u0101\u000b\r\u0003\r\u0003\r\u0007\r",
    "\u0105\n\r\f\r\u000e\r\u0108\u000b\r\u0003\r\u0003\r\u0007\r\u010c\n",
    "\r\f\r\u000e\r\u010f\u000b\r\u0003\r\u0003\r\u0007\r\u0113\n\r\f\r\u000e",
    "\r\u0116\u000b\r\u0003\r\u0003\r\u0003\u000e\u0003\u000e\u0007\u000e",
    "\u011c\n\u000e\f\u000e\u000e\u000e\u011f\u000b\u000e\u0003\u000e\u0003",
    "\u000e\u0007\u000e\u0123\n\u000e\f\u000e\u000e\u000e\u0126\u000b\u000e",
    "\u0003\u000e\u0003\u000e\u0003\u000f\u0003\u000f\u0003\u000f\u0003\u000f",
    "\u0005\u000f\u012e\n\u000f\u0003\u0010\u0007\u0010\u0131\n\u0010\f\u0010",
    "\u000e\u0010\u0134\u000b\u0010\u0003\u0010\u0003\u0010\u0003\u0010\u0007",
    "\u0010\u0139\n\u0010\f\u0010\u000e\u0010\u013c\u000b\u0010\u0003\u0010",
    "\u0007\u0010\u013f\n\u0010\f\u0010\u000e\u0010\u0142\u000b\u0010\u0003",
    "\u0010\u0007\u0010\u0145\n\u0010\f\u0010\u000e\u0010\u0148\u000b\u0010",
    "\u0003\u0011\u0003\u0011\u0007\u0011\u014c\n\u0011\f\u0011\u000e\u0011",
    "\u014f\u000b\u0011\u0003\u0011\u0003\u0011\u0005\u0011\u0153\n\u0011",
    "\u0003\u0011\u0003\u0011\u0003\u0012\u0003\u0012\u0007\u0012\u0159\n",
    "\u0012\f\u0012\u000e\u0012\u015c\u000b\u0012\u0003\u0012\u0003\u0012",
    "\u0007\u0012\u0160\n\u0012\f\u0012\u000e\u0012\u0163\u000b\u0012\u0003",
    "\u0012\u0005\u0012\u0166\n\u0012\u0003\u0013\u0003\u0013\u0003\u0014",
    "\u0003\u0014\u0003\u0014\u0003\u0014\u0007\u0014\u016e\n\u0014\f\u0014",
    "\u000e\u0014\u0171\u000b\u0014\u0003\u0014\u0003\u0014\u0003\u0014\u0005",
    "\u0014\u0176\n\u0014\u0003\u0015\u0003\u0015\u0006\u0015\u017a\n\u0015",
    "\r\u0015\u000e\u0015\u017b\u0003\u0015\u0003\u0015\u0006\u0015\u0180",
    "\n\u0015\r\u0015\u000e\u0015\u0181\u0003\u0015\u0003\u0015\u0003\u0015",
    "\u0002\u0002\u0016\u0002\u0004\u0006\b\n\f\u000e\u0010\u0012\u0014\u0016",
    "\u0018\u001a\u001c\u001e \"$&(\u0002\u0004\u0003\u0002\u0014\u0015\u0004",
    "\u0002\t\t\u0016\u0017\u0002\u01ad\u0002Q\u0003\u0002\u0002\u0002\u0004",
    "S\u0003\u0002\u0002\u0002\u0006U\u0003\u0002\u0002\u0002\bW\u0003\u0002",
    "\u0002\u0002\n\u0085\u0003\u0002\u0002\u0002\f\u0087\u0003\u0002\u0002",
    "\u0002\u000e\u00aa\u0003\u0002\u0002\u0002\u0010\u00c2\u0003\u0002\u0002",
    "\u0002\u0012\u00cb\u0003\u0002\u0002\u0002\u0014\u00cd\u0003\u0002\u0002",
    "\u0002\u0016\u00cf\u0003\u0002\u0002\u0002\u0018\u00f4\u0003\u0002\u0002",
    "\u0002\u001a\u0119\u0003\u0002\u0002\u0002\u001c\u012d\u0003\u0002\u0002",
    "\u0002\u001e\u0132\u0003\u0002\u0002\u0002 \u0149\u0003\u0002\u0002",
    "\u0002\"\u0156\u0003\u0002\u0002\u0002$\u0167\u0003\u0002\u0002\u0002",
    "&\u0169\u0003\u0002\u0002\u0002(\u0177\u0003\u0002\u0002\u0002*,\u0005",
    "\u0004\u0003\u0002+*\u0003\u0002\u0002\u0002,/\u0003\u0002\u0002\u0002",
    "-+\u0003\u0002\u0002\u0002-.\u0003\u0002\u0002\u0002.8\u0003\u0002\u0002",
    "\u0002/-\u0003\u0002\u0002\u000207\u0005\u0016\f\u000213\u0005\u0004",
    "\u0003\u000221\u0003\u0002\u0002\u000234\u0003\u0002\u0002\u000242\u0003",
    "\u0002\u0002\u000245\u0003\u0002\u0002\u000257\u0003\u0002\u0002\u0002",
    "60\u0003\u0002\u0002\u000262\u0003\u0002\u0002\u00027:\u0003\u0002\u0002",
    "\u000286\u0003\u0002\u0002\u000289\u0003\u0002\u0002\u00029C\u0003\u0002",
    "\u0002\u0002:8\u0003\u0002\u0002\u0002;B\u0005&\u0014\u0002<>\u0005",
    "\u0004\u0003\u0002=<\u0003\u0002\u0002\u0002>?\u0003\u0002\u0002\u0002",
    "?=\u0003\u0002\u0002\u0002?@\u0003\u0002\u0002\u0002@B\u0003\u0002\u0002",
    "\u0002A;\u0003\u0002\u0002\u0002A=\u0003\u0002\u0002\u0002BE\u0003\u0002",
    "\u0002\u0002CA\u0003\u0002\u0002\u0002CD\u0003\u0002\u0002\u0002DL\u0003",
    "\u0002\u0002\u0002EC\u0003\u0002\u0002\u0002FM\u0005(\u0015\u0002GI",
    "\u0005\u0004\u0003\u0002HG\u0003\u0002\u0002\u0002IJ\u0003\u0002\u0002",
    "\u0002JH\u0003\u0002\u0002\u0002JK\u0003\u0002\u0002\u0002KM\u0003\u0002",
    "\u0002\u0002LF\u0003\u0002\u0002\u0002LH\u0003\u0002\u0002\u0002MN\u0003",
    "\u0002\u0002\u0002NL\u0003\u0002\u0002\u0002NO\u0003\u0002\u0002\u0002",
    "OR\u0003\u0002\u0002\u0002PR\u0007\u0002\u0002\u0003Q-\u0003\u0002\u0002",
    "\u0002QP\u0003\u0002\u0002\u0002R\u0003\u0003\u0002\u0002\u0002ST\t",
    "\u0002\u0002\u0002T\u0005\u0003\u0002\u0002\u0002UV\u0007\u0018\u0002",
    "\u0002V\u0007\u0003\u0002\u0002\u0002W[\u0007\u000f\u0002\u0002XZ\u0005",
    "\u0004\u0003\u0002YX\u0003\u0002\u0002\u0002Z]\u0003\u0002\u0002\u0002",
    "[Y\u0003\u0002\u0002\u0002[\\\u0003\u0002\u0002\u0002\\^\u0003\u0002",
    "\u0002\u0002][\u0003\u0002\u0002\u0002^b\u0005\n\u0006\u0002_a\u0005",
    "\u0004\u0003\u0002`_\u0003\u0002\u0002\u0002ad\u0003\u0002\u0002\u0002",
    "b`\u0003\u0002\u0002\u0002bc\u0003\u0002\u0002\u0002cu\u0003\u0002\u0002",
    "\u0002db\u0003\u0002\u0002\u0002ei\u0007\n\u0002\u0002fh\u0005\u0004",
    "\u0003\u0002gf\u0003\u0002\u0002\u0002hk\u0003\u0002\u0002\u0002ig\u0003",
    "\u0002\u0002\u0002ij\u0003\u0002\u0002\u0002jl\u0003\u0002\u0002\u0002",
    "ki\u0003\u0002\u0002\u0002lp\u0005\n\u0006\u0002mo\u0005\u0004\u0003",
    "\u0002nm\u0003\u0002\u0002\u0002or\u0003\u0002\u0002\u0002pn\u0003\u0002",
    "\u0002\u0002pq\u0003\u0002\u0002\u0002qt\u0003\u0002\u0002\u0002rp\u0003",
    "\u0002\u0002\u0002se\u0003\u0002\u0002\u0002tw\u0003\u0002\u0002\u0002",
    "us\u0003\u0002\u0002\u0002uv\u0003\u0002\u0002\u0002vx\u0003\u0002\u0002",
    "\u0002wu\u0003\u0002\u0002\u0002xy\u0007\u0010\u0002\u0002y\t\u0003",
    "\u0002\u0002\u0002z~\u0005\u0006\u0004\u0002{}\u0005\u0004\u0003\u0002",
    "|{\u0003\u0002\u0002\u0002}\u0080\u0003\u0002\u0002\u0002~|\u0003\u0002",
    "\u0002\u0002~\u007f\u0003\u0002\u0002\u0002\u007f\u0082\u0003\u0002",
    "\u0002\u0002\u0080~\u0003\u0002\u0002\u0002\u0081\u0083\u0005\b\u0005",
    "\u0002\u0082\u0081\u0003\u0002\u0002\u0002\u0082\u0083\u0003\u0002\u0002",
    "\u0002\u0083\u0086\u0003\u0002\u0002\u0002\u0084\u0086\u0007\u0012\u0002",
    "\u0002\u0085z\u0003\u0002\u0002\u0002\u0085\u0084\u0003\u0002\u0002",
    "\u0002\u0086\u000b\u0003\u0002\u0002\u0002\u0087\u0089\u0007\u0003\u0002",
    "\u0002\u0088\u008a\u0005\u0004\u0003\u0002\u0089\u0088\u0003\u0002\u0002",
    "\u0002\u008a\u008b\u0003\u0002\u0002\u0002\u008b\u0089\u0003\u0002\u0002",
    "\u0002\u008b\u008c\u0003\u0002\u0002\u0002\u008c\u008d\u0003\u0002\u0002",
    "\u0002\u008d\u0091\u0007\r\u0002\u0002\u008e\u008f\u0007\u0018\u0002",
    "\u0002\u008f\u0090\u0007\u0013\u0002\u0002\u0090\u0092\u0005\n\u0006",
    "\u0002\u0091\u008e\u0003\u0002\u0002\u0002\u0091\u0092\u0003\u0002\u0002",
    "\u0002\u0092\u0093\u0003\u0002\u0002\u0002\u0093\u0097\u0007\u000e\u0002",
    "\u0002\u0094\u0096\u0005\u0004\u0003\u0002\u0095\u0094\u0003\u0002\u0002",
    "\u0002\u0096\u0099\u0003\u0002\u0002\u0002\u0097\u0095\u0003\u0002\u0002",
    "\u0002\u0097\u0098\u0003\u0002\u0002\u0002\u0098\u009a\u0003\u0002\u0002",
    "\u0002\u0099\u0097\u0003\u0002\u0002\u0002\u009a\u009e\u0007\u0013\u0002",
    "\u0002\u009b\u009d\u0005\u0004\u0003\u0002\u009c\u009b\u0003\u0002\u0002",
    "\u0002\u009d\u00a0\u0003\u0002\u0002\u0002\u009e\u009c\u0003\u0002\u0002",
    "\u0002\u009e\u009f\u0003\u0002\u0002\u0002\u009f\u00a1\u0003\u0002\u0002",
    "\u0002\u00a0\u009e\u0003\u0002\u0002\u0002\u00a1\u00a5\u0007\u0012\u0002",
    "\u0002\u00a2\u00a4\u0005\u0004\u0003\u0002\u00a3\u00a2\u0003\u0002\u0002",
    "\u0002\u00a4\u00a7\u0003\u0002\u0002\u0002\u00a5\u00a3\u0003\u0002\u0002",
    "\u0002\u00a5\u00a6\u0003\u0002\u0002\u0002\u00a6\u00a8\u0003\u0002\u0002",
    "\u0002\u00a7\u00a5\u0003\u0002\u0002\u0002\u00a8\u00a9\u0005\u000e\b",
    "\u0002\u00a9\r\u0003\u0002\u0002\u0002\u00aa\u00ae\u0007\u000b\u0002",
    "\u0002\u00ab\u00ad\u0005\u0004\u0003\u0002\u00ac\u00ab\u0003\u0002\u0002",
    "\u0002\u00ad\u00b0\u0003\u0002\u0002\u0002\u00ae\u00ac\u0003\u0002\u0002",
    "\u0002\u00ae\u00af\u0003\u0002\u0002\u0002\u00af\u00b2\u0003\u0002\u0002",
    "\u0002\u00b0\u00ae\u0003\u0002\u0002\u0002\u00b1\u00b3\u0005\u0010\t",
    "\u0002\u00b2\u00b1\u0003\u0002\u0002\u0002\u00b3\u00b4\u0003\u0002\u0002",
    "\u0002\u00b4\u00b2\u0003\u0002\u0002\u0002\u00b4\u00b5\u0003\u0002\u0002",
    "\u0002\u00b5\u00b9\u0003\u0002\u0002\u0002\u00b6\u00b8\u0005\u0004\u0003",
    "\u0002\u00b7\u00b6\u0003\u0002\u0002\u0002\u00b8\u00bb\u0003\u0002\u0002",
    "\u0002\u00b9\u00b7\u0003\u0002\u0002\u0002\u00b9\u00ba\u0003\u0002\u0002",
    "\u0002\u00ba\u00bc\u0003\u0002\u0002\u0002\u00bb\u00b9\u0003\u0002\u0002",
    "\u0002\u00bc\u00bd\u0007\f\u0002\u0002\u00bd\u000f\u0003\u0002\u0002",
    "\u0002\u00be\u00c3\u0005\u0012\n\u0002\u00bf\u00c3\u0005\u001a\u000e",
    "\u0002\u00c0\u00c3\u0005 \u0011\u0002\u00c1\u00c3\u0005\"\u0012\u0002",
    "\u00c2\u00be\u0003\u0002\u0002\u0002\u00c2\u00bf\u0003\u0002\u0002\u0002",
    "\u00c2\u00c0\u0003\u0002\u0002\u0002\u00c2\u00c1\u0003\u0002\u0002\u0002",
    "\u00c3\u00c5\u0003\u0002\u0002\u0002\u00c4\u00c6\u0005\u0004\u0003\u0002",
    "\u00c5\u00c4\u0003\u0002\u0002\u0002\u00c6\u00c7\u0003\u0002\u0002\u0002",
    "\u00c7\u00c5\u0003\u0002\u0002\u0002\u00c7\u00c8\u0003\u0002\u0002\u0002",
    "\u00c8\u0011\u0003\u0002\u0002\u0002\u00c9\u00cc\u0005\u0016\f\u0002",
    "\u00ca\u00cc\u0005\u0018\r\u0002\u00cb\u00c9\u0003\u0002\u0002\u0002",
    "\u00cb\u00ca\u0003\u0002\u0002\u0002\u00cc\u0013\u0003\u0002\u0002\u0002",
    "\u00cd\u00ce\u0007\u0018\u0002\u0002\u00ce\u0015\u0003\u0002\u0002\u0002",
    "\u00cf\u00d3\u0007\u0006\u0002\u0002\u00d0\u00d2\u0005\u0004\u0003\u0002",
    "\u00d1\u00d0\u0003\u0002\u0002\u0002\u00d2\u00d5\u0003\u0002\u0002\u0002",
    "\u00d3\u00d1\u0003\u0002\u0002\u0002\u00d3\u00d4\u0003\u0002\u0002\u0002",
    "\u00d4\u00d6\u0003\u0002\u0002\u0002\u00d5\u00d3\u0003\u0002\u0002\u0002",
    "\u00d6\u00da\u0005\u0014\u000b\u0002\u00d7\u00d9\u0005\u0004\u0003\u0002",
    "\u00d8\u00d7\u0003\u0002\u0002\u0002\u00d9\u00dc\u0003\u0002\u0002\u0002",
    "\u00da\u00d8\u0003\u0002\u0002\u0002\u00da\u00db\u0003\u0002\u0002\u0002",
    "\u00db\u00dd\u0003\u0002\u0002\u0002\u00dc\u00da\u0003\u0002\u0002\u0002",
    "\u00dd\u00e1\u0007\u0013\u0002\u0002\u00de\u00e0\u0005\u0004\u0003\u0002",
    "\u00df\u00de\u0003\u0002\u0002\u0002\u00e0\u00e3\u0003\u0002\u0002\u0002",
    "\u00e1\u00df\u0003\u0002\u0002\u0002\u00e1\u00e2\u0003\u0002\u0002\u0002",
    "\u00e2\u00e4\u0003\u0002\u0002\u0002\u00e3\u00e1\u0003\u0002\u0002\u0002",
    "\u00e4\u00e8\u0005\n\u0006\u0002\u00e5\u00e7\u0005\u0004\u0003\u0002",
    "\u00e6\u00e5\u0003\u0002\u0002\u0002\u00e7\u00ea\u0003\u0002\u0002\u0002",
    "\u00e8\u00e6\u0003\u0002\u0002\u0002\u00e8\u00e9\u0003\u0002\u0002\u0002",
    "\u00e9\u00eb\u0003\u0002\u0002\u0002\u00ea\u00e8\u0003\u0002\u0002\u0002",
    "\u00eb\u00ef\u0007\u0011\u0002\u0002\u00ec\u00ee\u0005\u0004\u0003\u0002",
    "\u00ed\u00ec\u0003\u0002\u0002\u0002\u00ee\u00f1\u0003\u0002\u0002\u0002",
    "\u00ef\u00ed\u0003\u0002\u0002\u0002\u00ef\u00f0\u0003\u0002\u0002\u0002",
    "\u00f0\u00f2\u0003\u0002\u0002\u0002\u00f1\u00ef\u0003\u0002\u0002\u0002",
    "\u00f2\u00f3\u0005\u001c\u000f\u0002\u00f3\u0017\u0003\u0002\u0002\u0002",
    "\u00f4\u00f8\u0007\u0007\u0002\u0002\u00f5\u00f7\u0005\u0004\u0003\u0002",
    "\u00f6\u00f5\u0003\u0002\u0002\u0002\u00f7\u00fa\u0003\u0002\u0002\u0002",
    "\u00f8\u00f6\u0003\u0002\u0002\u0002\u00f8\u00f9\u0003\u0002\u0002\u0002",
    "\u00f9\u00fb\u0003\u0002\u0002\u0002\u00fa\u00f8\u0003\u0002\u0002\u0002",
    "\u00fb\u00ff\u0005\u0014\u000b\u0002\u00fc\u00fe\u0005\u0004\u0003\u0002",
    "\u00fd\u00fc\u0003\u0002\u0002\u0002\u00fe\u0101\u0003\u0002\u0002\u0002",
    "\u00ff\u00fd\u0003\u0002\u0002\u0002\u00ff\u0100\u0003\u0002\u0002\u0002",
    "\u0100\u0102\u0003\u0002\u0002\u0002\u0101\u00ff\u0003\u0002\u0002\u0002",
    "\u0102\u0106\u0007\u0013\u0002\u0002\u0103\u0105\u0005\u0004\u0003\u0002",
    "\u0104\u0103\u0003\u0002\u0002\u0002\u0105\u0108\u0003\u0002\u0002\u0002",
    "\u0106\u0104\u0003\u0002\u0002\u0002\u0106\u0107\u0003\u0002\u0002\u0002",
    "\u0107\u0109\u0003\u0002\u0002\u0002\u0108\u0106\u0003\u0002\u0002\u0002",
    "\u0109\u010d\u0005\n\u0006\u0002\u010a\u010c\u0005\u0004\u0003\u0002",
    "\u010b\u010a\u0003\u0002\u0002\u0002\u010c\u010f\u0003\u0002\u0002\u0002",
    "\u010d\u010b\u0003\u0002\u0002\u0002\u010d\u010e\u0003\u0002\u0002\u0002",
    "\u010e\u0110\u0003\u0002\u0002\u0002\u010f\u010d\u0003\u0002\u0002\u0002",
    "\u0110\u0114\u0007\u0011\u0002\u0002\u0111\u0113\u0005\u0004\u0003\u0002",
    "\u0112\u0111\u0003\u0002\u0002\u0002\u0113\u0116\u0003\u0002\u0002\u0002",
    "\u0114\u0112\u0003\u0002\u0002\u0002\u0114\u0115\u0003\u0002\u0002\u0002",
    "\u0115\u0117\u0003\u0002\u0002\u0002\u0116\u0114\u0003\u0002\u0002\u0002",
    "\u0117\u0118\u0005\u001c\u000f\u0002\u0118\u0019\u0003\u0002\u0002\u0002",
    "\u0119\u011d\u0005\u0014\u000b\u0002\u011a\u011c\u0005\u0004\u0003\u0002",
    "\u011b\u011a\u0003\u0002\u0002\u0002\u011c\u011f\u0003\u0002\u0002\u0002",
    "\u011d\u011b\u0003\u0002\u0002\u0002\u011d\u011e\u0003\u0002\u0002\u0002",
    "\u011e\u0120\u0003\u0002\u0002\u0002\u011f\u011d\u0003\u0002\u0002\u0002",
    "\u0120\u0124\u0007\u0011\u0002\u0002\u0121\u0123\u0005\u0004\u0003\u0002",
    "\u0122\u0121\u0003\u0002\u0002\u0002\u0123\u0126\u0003\u0002\u0002\u0002",
    "\u0124\u0122\u0003\u0002\u0002\u0002\u0124\u0125\u0003\u0002\u0002\u0002",
    "\u0125\u0127\u0003\u0002\u0002\u0002\u0126\u0124\u0003\u0002\u0002\u0002",
    "\u0127\u0128\u0005\u001c\u000f\u0002\u0128\u001b\u0003\u0002\u0002\u0002",
    "\u0129\u012e\u0005\f\u0007\u0002\u012a\u012e\u0005 \u0011\u0002\u012b",
    "\u012e\u0005$\u0013\u0002\u012c\u012e\u0007\u0018\u0002\u0002\u012d",
    "\u0129\u0003\u0002\u0002\u0002\u012d\u012a\u0003\u0002\u0002\u0002\u012d",
    "\u012b\u0003\u0002\u0002\u0002\u012d\u012c\u0003\u0002\u0002\u0002\u012e",
    "\u001d\u0003\u0002\u0002\u0002\u012f\u0131\u0005\u0004\u0003\u0002\u0130",
    "\u012f\u0003\u0002\u0002\u0002\u0131\u0134\u0003\u0002\u0002\u0002\u0132",
    "\u0130\u0003\u0002\u0002\u0002\u0132\u0133\u0003\u0002\u0002\u0002\u0133",
    "\u0135\u0003\u0002\u0002\u0002\u0134\u0132\u0003\u0002\u0002\u0002\u0135",
    "\u0140\u0007\u0018\u0002\u0002\u0136\u013a\u0007\n\u0002\u0002\u0137",
    "\u0139\u0005\u0004\u0003\u0002\u0138\u0137\u0003\u0002\u0002\u0002\u0139",
    "\u013c\u0003\u0002\u0002\u0002\u013a\u0138\u0003\u0002\u0002\u0002\u013a",
    "\u013b\u0003\u0002\u0002\u0002\u013b\u013d\u0003\u0002\u0002\u0002\u013c",
    "\u013a\u0003\u0002\u0002\u0002\u013d\u013f\u0007\u0018\u0002\u0002\u013e",
    "\u0136\u0003\u0002\u0002\u0002\u013f\u0142\u0003\u0002\u0002\u0002\u0140",
    "\u013e\u0003\u0002\u0002\u0002\u0140\u0141\u0003\u0002\u0002\u0002\u0141",
    "\u0146\u0003\u0002\u0002\u0002\u0142\u0140\u0003\u0002\u0002\u0002\u0143",
    "\u0145\u0005\u0004\u0003\u0002\u0144\u0143\u0003\u0002\u0002\u0002\u0145",
    "\u0148\u0003\u0002\u0002\u0002\u0146\u0144\u0003\u0002\u0002\u0002\u0146",
    "\u0147\u0003\u0002\u0002\u0002\u0147\u001f\u0003\u0002\u0002\u0002\u0148",
    "\u0146\u0003\u0002\u0002\u0002\u0149\u014d\u0007\u0018\u0002\u0002\u014a",
    "\u014c\u0007\u0015\u0002\u0002\u014b\u014a\u0003\u0002\u0002\u0002\u014c",
    "\u014f\u0003\u0002\u0002\u0002\u014d\u014b\u0003\u0002\u0002\u0002\u014d",
    "\u014e\u0003\u0002\u0002\u0002\u014e\u0150\u0003\u0002\u0002\u0002\u014f",
    "\u014d\u0003\u0002\u0002\u0002\u0150\u0152\u0007\r\u0002\u0002\u0151",
    "\u0153\u0005\u001e\u0010\u0002\u0152\u0151\u0003\u0002\u0002\u0002\u0152",
    "\u0153\u0003\u0002\u0002\u0002\u0153\u0154\u0003\u0002\u0002\u0002\u0154",
    "\u0155\u0007\u000e\u0002\u0002\u0155!\u0003\u0002\u0002\u0002\u0156",
    "\u015a\u0007\b\u0002\u0002\u0157\u0159\u0005\u0004\u0003\u0002\u0158",
    "\u0157\u0003\u0002\u0002\u0002\u0159\u015c\u0003\u0002\u0002\u0002\u015a",
    "\u0158\u0003\u0002\u0002\u0002\u015a\u015b\u0003\u0002\u0002\u0002\u015b",
    "\u015d\u0003\u0002\u0002\u0002\u015c\u015a\u0003\u0002\u0002\u0002\u015d",
    "\u0165\u0007\u0018\u0002\u0002\u015e\u0160\u0005\u0004\u0003\u0002\u015f",
    "\u015e\u0003\u0002\u0002\u0002\u0160\u0163\u0003\u0002\u0002\u0002\u0161",
    "\u015f\u0003\u0002\u0002\u0002\u0161\u0162\u0003\u0002\u0002\u0002\u0162",
    "\u0164\u0003\u0002\u0002\u0002\u0163\u0161\u0003\u0002\u0002\u0002\u0164",
    "\u0166\u0007\u0018\u0002\u0002\u0165\u0161\u0003\u0002\u0002\u0002\u0165",
    "\u0166\u0003\u0002\u0002\u0002\u0166#\u0003\u0002\u0002\u0002\u0167",
    "\u0168\t\u0003\u0002\u0002\u0168%\u0003\u0002\u0002\u0002\u0169\u016a",
    "\u0007\u0004\u0002\u0002\u016a\u016b\u0005\u0004\u0003\u0002\u016b\u016f",
    "\u0007\u0018\u0002\u0002\u016c\u016e\u0005\u0004\u0003\u0002\u016d\u016c",
    "\u0003\u0002\u0002\u0002\u016e\u0171\u0003\u0002\u0002\u0002\u016f\u016d",
    "\u0003\u0002\u0002\u0002\u016f\u0170\u0003\u0002\u0002\u0002\u0170\u0172",
    "\u0003\u0002\u0002\u0002\u0171\u016f\u0003\u0002\u0002\u0002\u0172\u0175",
    "\u0007\u0013\u0002\u0002\u0173\u0176\u0005\u0006\u0004\u0002\u0174\u0176",
    "\u0007\u0012\u0002\u0002\u0175\u0173\u0003\u0002\u0002\u0002\u0175\u0174",
    "\u0003\u0002\u0002\u0002\u0176\'\u0003\u0002\u0002\u0002\u0177\u0179",
    "\u0007\u0005\u0002\u0002\u0178\u017a\u0005\u0004\u0003\u0002\u0179\u0178",
    "\u0003\u0002\u0002\u0002\u017a\u017b\u0003\u0002\u0002\u0002\u017b\u0179",
    "\u0003\u0002\u0002\u0002\u017b\u017c\u0003\u0002\u0002\u0002\u017c\u017d",
    "\u0003\u0002\u0002\u0002\u017d\u017f\u0007\u0018\u0002\u0002\u017e\u0180",
    "\u0005\u0004\u0003\u0002\u017f\u017e\u0003\u0002\u0002\u0002\u0180\u0181",
    "\u0003\u0002\u0002\u0002\u0181\u017f\u0003\u0002\u0002\u0002\u0181\u0182",
    "\u0003\u0002\u0002\u0002\u0182\u0183\u0003\u0002\u0002\u0002\u0183\u0184",
    "\u0005\f\u0007\u0002\u0184)\u0003\u0002\u0002\u0002:-468?ACJLNQ[bip",
    "u~\u0082\u0085\u008b\u0091\u0097\u009e\u00a5\u00ae\u00b4\u00b9\u00c2",
    "\u00c7\u00cb\u00d3\u00da\u00e1\u00e8\u00ef\u00f8\u00ff\u0106\u010d\u0114",
    "\u011d\u0124\u012d\u0132\u013a\u0140\u0146\u014d\u0152\u015a\u0161\u0165",
    "\u016f\u0175\u017b\u0181"].join("");


var atn = new antlr4.atn.ATNDeserializer().deserialize(serializedATN);

var decisionsToDFA = atn.decisionToState.map( function(ds, index) { return new antlr4.dfa.DFA(ds, index); });

var sharedContextCache = new antlr4.PredictionContextCache();

var literalNames = [ null, "'fn'", "'event'", "'on'", "'const'", "'let'", 
                     "'emit'", null, null, "'{'", "'}'", "'('", "')'", "'<'", 
                     "'>'", "'='", "'void'" ];

var symbolicNames = [ null, "FN", "EVENT", "ON", "CONST", "LET", "EMIT", 
                      "BOOLCONSTANT", "SEP", "OPENBODY", "CLOSEBODY", "OPENARGS", 
                      "CLOSEARGS", "OPENGENERIC", "CLOSEGENERIC", "EQUALS", 
                      "VOID", "TYPESEP", "NEWLINE", "WS", "STRINGCONSTANT", 
                      "NUMBERCONSTANT", "VARNAME" ];

var ruleNames =  [ "module", "blank", "typename", "typegenerics", "fulltypename", 
                   "functions", "functionbody", "statements", "declarations", 
                   "decname", "constdeclaration", "letdeclaration", "assignments", 
                   "assignables", "calllist", "calls", "emits", "constants", 
                   "events", "handlers" ];

function AmmParser (input) {
	antlr4.Parser.call(this, input);
    this._interp = new antlr4.atn.ParserATNSimulator(this, atn, decisionsToDFA, sharedContextCache);
    this.ruleNames = ruleNames;
    this.literalNames = literalNames;
    this.symbolicNames = symbolicNames;
    return this;
}

AmmParser.prototype = Object.create(antlr4.Parser.prototype);
AmmParser.prototype.constructor = AmmParser;

Object.defineProperty(AmmParser.prototype, "atn", {
	get : function() {
		return atn;
	}
});

AmmParser.EOF = antlr4.Token.EOF;
AmmParser.FN = 1;
AmmParser.EVENT = 2;
AmmParser.ON = 3;
AmmParser.CONST = 4;
AmmParser.LET = 5;
AmmParser.EMIT = 6;
AmmParser.BOOLCONSTANT = 7;
AmmParser.SEP = 8;
AmmParser.OPENBODY = 9;
AmmParser.CLOSEBODY = 10;
AmmParser.OPENARGS = 11;
AmmParser.CLOSEARGS = 12;
AmmParser.OPENGENERIC = 13;
AmmParser.CLOSEGENERIC = 14;
AmmParser.EQUALS = 15;
AmmParser.VOID = 16;
AmmParser.TYPESEP = 17;
AmmParser.NEWLINE = 18;
AmmParser.WS = 19;
AmmParser.STRINGCONSTANT = 20;
AmmParser.NUMBERCONSTANT = 21;
AmmParser.VARNAME = 22;

AmmParser.RULE_module = 0;
AmmParser.RULE_blank = 1;
AmmParser.RULE_typename = 2;
AmmParser.RULE_typegenerics = 3;
AmmParser.RULE_fulltypename = 4;
AmmParser.RULE_functions = 5;
AmmParser.RULE_functionbody = 6;
AmmParser.RULE_statements = 7;
AmmParser.RULE_declarations = 8;
AmmParser.RULE_decname = 9;
AmmParser.RULE_constdeclaration = 10;
AmmParser.RULE_letdeclaration = 11;
AmmParser.RULE_assignments = 12;
AmmParser.RULE_assignables = 13;
AmmParser.RULE_calllist = 14;
AmmParser.RULE_calls = 15;
AmmParser.RULE_emits = 16;
AmmParser.RULE_constants = 17;
AmmParser.RULE_events = 18;
AmmParser.RULE_handlers = 19;


function ModuleContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_module;
    return this;
}

ModuleContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
ModuleContext.prototype.constructor = ModuleContext;

ModuleContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

ModuleContext.prototype.constdeclaration = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(ConstdeclarationContext);
    } else {
        return this.getTypedRuleContext(ConstdeclarationContext,i);
    }
};

ModuleContext.prototype.events = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(EventsContext);
    } else {
        return this.getTypedRuleContext(EventsContext,i);
    }
};

ModuleContext.prototype.handlers = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(HandlersContext);
    } else {
        return this.getTypedRuleContext(HandlersContext,i);
    }
};

ModuleContext.prototype.EOF = function() {
    return this.getToken(AmmParser.EOF, 0);
};

ModuleContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterModule(this);
	}
};

ModuleContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitModule(this);
	}
};




AmmParser.ModuleContext = ModuleContext;

AmmParser.prototype.module = function() {

    var localctx = new ModuleContext(this, this._ctx, this.state);
    this.enterRule(localctx, 0, AmmParser.RULE_module);
    var _la = 0; // Token type
    try {
        this.state = 79;
        this._errHandler.sync(this);
        switch(this._input.LA(1)) {
        case AmmParser.EVENT:
        case AmmParser.ON:
        case AmmParser.CONST:
        case AmmParser.NEWLINE:
        case AmmParser.WS:
            this.enterOuterAlt(localctx, 1);
            this.state = 43;
            this._errHandler.sync(this);
            var _alt = this._interp.adaptivePredict(this._input,0,this._ctx)
            while(_alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER) {
                if(_alt===1) {
                    this.state = 40;
                    this.blank(); 
                }
                this.state = 45;
                this._errHandler.sync(this);
                _alt = this._interp.adaptivePredict(this._input,0,this._ctx);
            }

            this.state = 54;
            this._errHandler.sync(this);
            var _alt = this._interp.adaptivePredict(this._input,3,this._ctx)
            while(_alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER) {
                if(_alt===1) {
                    this.state = 52;
                    this._errHandler.sync(this);
                    switch(this._input.LA(1)) {
                    case AmmParser.CONST:
                        this.state = 46;
                        this.constdeclaration();
                        break;
                    case AmmParser.NEWLINE:
                    case AmmParser.WS:
                        this.state = 48; 
                        this._errHandler.sync(this);
                        var _alt = 1;
                        do {
                        	switch (_alt) {
                        	case 1:
                        		this.state = 47;
                        		this.blank();
                        		break;
                        	default:
                        		throw new antlr4.error.NoViableAltException(this);
                        	}
                        	this.state = 50; 
                        	this._errHandler.sync(this);
                        	_alt = this._interp.adaptivePredict(this._input,1, this._ctx);
                        } while ( _alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER );
                        break;
                    default:
                        throw new antlr4.error.NoViableAltException(this);
                    } 
                }
                this.state = 56;
                this._errHandler.sync(this);
                _alt = this._interp.adaptivePredict(this._input,3,this._ctx);
            }

            this.state = 65;
            this._errHandler.sync(this);
            var _alt = this._interp.adaptivePredict(this._input,6,this._ctx)
            while(_alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER) {
                if(_alt===1) {
                    this.state = 63;
                    this._errHandler.sync(this);
                    switch(this._input.LA(1)) {
                    case AmmParser.EVENT:
                        this.state = 57;
                        this.events();
                        break;
                    case AmmParser.NEWLINE:
                    case AmmParser.WS:
                        this.state = 59; 
                        this._errHandler.sync(this);
                        var _alt = 1;
                        do {
                        	switch (_alt) {
                        	case 1:
                        		this.state = 58;
                        		this.blank();
                        		break;
                        	default:
                        		throw new antlr4.error.NoViableAltException(this);
                        	}
                        	this.state = 61; 
                        	this._errHandler.sync(this);
                        	_alt = this._interp.adaptivePredict(this._input,4, this._ctx);
                        } while ( _alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER );
                        break;
                    default:
                        throw new antlr4.error.NoViableAltException(this);
                    } 
                }
                this.state = 67;
                this._errHandler.sync(this);
                _alt = this._interp.adaptivePredict(this._input,6,this._ctx);
            }

            this.state = 74; 
            this._errHandler.sync(this);
            _la = this._input.LA(1);
            do {
                this.state = 74;
                this._errHandler.sync(this);
                switch(this._input.LA(1)) {
                case AmmParser.ON:
                    this.state = 68;
                    this.handlers();
                    break;
                case AmmParser.NEWLINE:
                case AmmParser.WS:
                    this.state = 70; 
                    this._errHandler.sync(this);
                    var _alt = 1;
                    do {
                    	switch (_alt) {
                    	case 1:
                    		this.state = 69;
                    		this.blank();
                    		break;
                    	default:
                    		throw new antlr4.error.NoViableAltException(this);
                    	}
                    	this.state = 72; 
                    	this._errHandler.sync(this);
                    	_alt = this._interp.adaptivePredict(this._input,7, this._ctx);
                    } while ( _alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER );
                    break;
                default:
                    throw new antlr4.error.NoViableAltException(this);
                }
                this.state = 76; 
                this._errHandler.sync(this);
                _la = this._input.LA(1);
            } while((((_la) & ~0x1f) == 0 && ((1 << _la) & ((1 << AmmParser.ON) | (1 << AmmParser.NEWLINE) | (1 << AmmParser.WS))) !== 0));
            break;
        case AmmParser.EOF:
            this.enterOuterAlt(localctx, 2);
            this.state = 78;
            this.match(AmmParser.EOF);
            break;
        default:
            throw new antlr4.error.NoViableAltException(this);
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function BlankContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_blank;
    return this;
}

BlankContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
BlankContext.prototype.constructor = BlankContext;

BlankContext.prototype.WS = function() {
    return this.getToken(AmmParser.WS, 0);
};

BlankContext.prototype.NEWLINE = function() {
    return this.getToken(AmmParser.NEWLINE, 0);
};

BlankContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterBlank(this);
	}
};

BlankContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitBlank(this);
	}
};




AmmParser.BlankContext = BlankContext;

AmmParser.prototype.blank = function() {

    var localctx = new BlankContext(this, this._ctx, this.state);
    this.enterRule(localctx, 2, AmmParser.RULE_blank);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 81;
        _la = this._input.LA(1);
        if(!(_la===AmmParser.NEWLINE || _la===AmmParser.WS)) {
        this._errHandler.recoverInline(this);
        }
        else {
        	this._errHandler.reportMatch(this);
            this.consume();
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function TypenameContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_typename;
    return this;
}

TypenameContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
TypenameContext.prototype.constructor = TypenameContext;

TypenameContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

TypenameContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterTypename(this);
	}
};

TypenameContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitTypename(this);
	}
};




AmmParser.TypenameContext = TypenameContext;

AmmParser.prototype.typename = function() {

    var localctx = new TypenameContext(this, this._ctx, this.state);
    this.enterRule(localctx, 4, AmmParser.RULE_typename);
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 83;
        this.match(AmmParser.VARNAME);
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function TypegenericsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_typegenerics;
    return this;
}

TypegenericsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
TypegenericsContext.prototype.constructor = TypegenericsContext;

TypegenericsContext.prototype.OPENGENERIC = function() {
    return this.getToken(AmmParser.OPENGENERIC, 0);
};

TypegenericsContext.prototype.fulltypename = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(FulltypenameContext);
    } else {
        return this.getTypedRuleContext(FulltypenameContext,i);
    }
};

TypegenericsContext.prototype.CLOSEGENERIC = function() {
    return this.getToken(AmmParser.CLOSEGENERIC, 0);
};

TypegenericsContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

TypegenericsContext.prototype.SEP = function(i) {
	if(i===undefined) {
		i = null;
	}
    if(i===null) {
        return this.getTokens(AmmParser.SEP);
    } else {
        return this.getToken(AmmParser.SEP, i);
    }
};


TypegenericsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterTypegenerics(this);
	}
};

TypegenericsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitTypegenerics(this);
	}
};




AmmParser.TypegenericsContext = TypegenericsContext;

AmmParser.prototype.typegenerics = function() {

    var localctx = new TypegenericsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 6, AmmParser.RULE_typegenerics);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 85;
        this.match(AmmParser.OPENGENERIC);
        this.state = 89;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 86;
            this.blank();
            this.state = 91;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 92;
        this.fulltypename();
        this.state = 96;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 93;
            this.blank();
            this.state = 98;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 115;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.SEP) {
            this.state = 99;
            this.match(AmmParser.SEP);
            this.state = 103;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
            while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
                this.state = 100;
                this.blank();
                this.state = 105;
                this._errHandler.sync(this);
                _la = this._input.LA(1);
            }
            this.state = 106;
            this.fulltypename();
            this.state = 110;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
            while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
                this.state = 107;
                this.blank();
                this.state = 112;
                this._errHandler.sync(this);
                _la = this._input.LA(1);
            }
            this.state = 117;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 118;
        this.match(AmmParser.CLOSEGENERIC);
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function FulltypenameContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_fulltypename;
    return this;
}

FulltypenameContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
FulltypenameContext.prototype.constructor = FulltypenameContext;

FulltypenameContext.prototype.typename = function() {
    return this.getTypedRuleContext(TypenameContext,0);
};

FulltypenameContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

FulltypenameContext.prototype.typegenerics = function() {
    return this.getTypedRuleContext(TypegenericsContext,0);
};

FulltypenameContext.prototype.VOID = function() {
    return this.getToken(AmmParser.VOID, 0);
};

FulltypenameContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterFulltypename(this);
	}
};

FulltypenameContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitFulltypename(this);
	}
};




AmmParser.FulltypenameContext = FulltypenameContext;

AmmParser.prototype.fulltypename = function() {

    var localctx = new FulltypenameContext(this, this._ctx, this.state);
    this.enterRule(localctx, 8, AmmParser.RULE_fulltypename);
    var _la = 0; // Token type
    try {
        this.state = 131;
        this._errHandler.sync(this);
        switch(this._input.LA(1)) {
        case AmmParser.VARNAME:
            this.enterOuterAlt(localctx, 1);
            this.state = 120;
            this.typename();
            this.state = 124;
            this._errHandler.sync(this);
            var _alt = this._interp.adaptivePredict(this._input,16,this._ctx)
            while(_alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER) {
                if(_alt===1) {
                    this.state = 121;
                    this.blank(); 
                }
                this.state = 126;
                this._errHandler.sync(this);
                _alt = this._interp.adaptivePredict(this._input,16,this._ctx);
            }

            this.state = 128;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
            if(_la===AmmParser.OPENGENERIC) {
                this.state = 127;
                this.typegenerics();
            }

            break;
        case AmmParser.VOID:
            this.enterOuterAlt(localctx, 2);
            this.state = 130;
            this.match(AmmParser.VOID);
            break;
        default:
            throw new antlr4.error.NoViableAltException(this);
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function FunctionsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_functions;
    return this;
}

FunctionsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
FunctionsContext.prototype.constructor = FunctionsContext;

FunctionsContext.prototype.FN = function() {
    return this.getToken(AmmParser.FN, 0);
};

FunctionsContext.prototype.OPENARGS = function() {
    return this.getToken(AmmParser.OPENARGS, 0);
};

FunctionsContext.prototype.CLOSEARGS = function() {
    return this.getToken(AmmParser.CLOSEARGS, 0);
};

FunctionsContext.prototype.TYPESEP = function(i) {
	if(i===undefined) {
		i = null;
	}
    if(i===null) {
        return this.getTokens(AmmParser.TYPESEP);
    } else {
        return this.getToken(AmmParser.TYPESEP, i);
    }
};


FunctionsContext.prototype.VOID = function() {
    return this.getToken(AmmParser.VOID, 0);
};

FunctionsContext.prototype.functionbody = function() {
    return this.getTypedRuleContext(FunctionbodyContext,0);
};

FunctionsContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

FunctionsContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

FunctionsContext.prototype.fulltypename = function() {
    return this.getTypedRuleContext(FulltypenameContext,0);
};

FunctionsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterFunctions(this);
	}
};

FunctionsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitFunctions(this);
	}
};




AmmParser.FunctionsContext = FunctionsContext;

AmmParser.prototype.functions = function() {

    var localctx = new FunctionsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 10, AmmParser.RULE_functions);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 133;
        this.match(AmmParser.FN);
        this.state = 135; 
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        do {
            this.state = 134;
            this.blank();
            this.state = 137; 
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        } while(_la===AmmParser.NEWLINE || _la===AmmParser.WS);
        this.state = 139;
        this.match(AmmParser.OPENARGS);
        this.state = 143;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        if(_la===AmmParser.VARNAME) {
            this.state = 140;
            this.match(AmmParser.VARNAME);
            this.state = 141;
            this.match(AmmParser.TYPESEP);
            this.state = 142;
            this.fulltypename();
        }

        this.state = 145;
        this.match(AmmParser.CLOSEARGS);
        this.state = 149;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 146;
            this.blank();
            this.state = 151;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 152;
        this.match(AmmParser.TYPESEP);
        this.state = 156;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 153;
            this.blank();
            this.state = 158;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 159;
        this.match(AmmParser.VOID);
        this.state = 163;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 160;
            this.blank();
            this.state = 165;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 166;
        this.functionbody();
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function FunctionbodyContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_functionbody;
    return this;
}

FunctionbodyContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
FunctionbodyContext.prototype.constructor = FunctionbodyContext;

FunctionbodyContext.prototype.OPENBODY = function() {
    return this.getToken(AmmParser.OPENBODY, 0);
};

FunctionbodyContext.prototype.CLOSEBODY = function() {
    return this.getToken(AmmParser.CLOSEBODY, 0);
};

FunctionbodyContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

FunctionbodyContext.prototype.statements = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(StatementsContext);
    } else {
        return this.getTypedRuleContext(StatementsContext,i);
    }
};

FunctionbodyContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterFunctionbody(this);
	}
};

FunctionbodyContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitFunctionbody(this);
	}
};




AmmParser.FunctionbodyContext = FunctionbodyContext;

AmmParser.prototype.functionbody = function() {

    var localctx = new FunctionbodyContext(this, this._ctx, this.state);
    this.enterRule(localctx, 12, AmmParser.RULE_functionbody);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 168;
        this.match(AmmParser.OPENBODY);
        this.state = 172;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 169;
            this.blank();
            this.state = 174;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 176; 
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        do {
            this.state = 175;
            this.statements();
            this.state = 178; 
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        } while((((_la) & ~0x1f) == 0 && ((1 << _la) & ((1 << AmmParser.CONST) | (1 << AmmParser.LET) | (1 << AmmParser.EMIT) | (1 << AmmParser.VARNAME))) !== 0));
        this.state = 183;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 180;
            this.blank();
            this.state = 185;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 186;
        this.match(AmmParser.CLOSEBODY);
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function StatementsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_statements;
    return this;
}

StatementsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
StatementsContext.prototype.constructor = StatementsContext;

StatementsContext.prototype.declarations = function() {
    return this.getTypedRuleContext(DeclarationsContext,0);
};

StatementsContext.prototype.assignments = function() {
    return this.getTypedRuleContext(AssignmentsContext,0);
};

StatementsContext.prototype.calls = function() {
    return this.getTypedRuleContext(CallsContext,0);
};

StatementsContext.prototype.emits = function() {
    return this.getTypedRuleContext(EmitsContext,0);
};

StatementsContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

StatementsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterStatements(this);
	}
};

StatementsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitStatements(this);
	}
};




AmmParser.StatementsContext = StatementsContext;

AmmParser.prototype.statements = function() {

    var localctx = new StatementsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 14, AmmParser.RULE_statements);
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 192;
        this._errHandler.sync(this);
        var la_ = this._interp.adaptivePredict(this._input,27,this._ctx);
        switch(la_) {
        case 1:
            this.state = 188;
            this.declarations();
            break;

        case 2:
            this.state = 189;
            this.assignments();
            break;

        case 3:
            this.state = 190;
            this.calls();
            break;

        case 4:
            this.state = 191;
            this.emits();
            break;

        }
        this.state = 195; 
        this._errHandler.sync(this);
        var _alt = 1;
        do {
        	switch (_alt) {
        	case 1:
        		this.state = 194;
        		this.blank();
        		break;
        	default:
        		throw new antlr4.error.NoViableAltException(this);
        	}
        	this.state = 197; 
        	this._errHandler.sync(this);
        	_alt = this._interp.adaptivePredict(this._input,28, this._ctx);
        } while ( _alt!=2 && _alt!=antlr4.atn.ATN.INVALID_ALT_NUMBER );
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function DeclarationsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_declarations;
    return this;
}

DeclarationsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
DeclarationsContext.prototype.constructor = DeclarationsContext;

DeclarationsContext.prototype.constdeclaration = function() {
    return this.getTypedRuleContext(ConstdeclarationContext,0);
};

DeclarationsContext.prototype.letdeclaration = function() {
    return this.getTypedRuleContext(LetdeclarationContext,0);
};

DeclarationsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterDeclarations(this);
	}
};

DeclarationsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitDeclarations(this);
	}
};




AmmParser.DeclarationsContext = DeclarationsContext;

AmmParser.prototype.declarations = function() {

    var localctx = new DeclarationsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 16, AmmParser.RULE_declarations);
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 201;
        this._errHandler.sync(this);
        switch(this._input.LA(1)) {
        case AmmParser.CONST:
            this.state = 199;
            this.constdeclaration();
            break;
        case AmmParser.LET:
            this.state = 200;
            this.letdeclaration();
            break;
        default:
            throw new antlr4.error.NoViableAltException(this);
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function DecnameContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_decname;
    return this;
}

DecnameContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
DecnameContext.prototype.constructor = DecnameContext;

DecnameContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

DecnameContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterDecname(this);
	}
};

DecnameContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitDecname(this);
	}
};




AmmParser.DecnameContext = DecnameContext;

AmmParser.prototype.decname = function() {

    var localctx = new DecnameContext(this, this._ctx, this.state);
    this.enterRule(localctx, 18, AmmParser.RULE_decname);
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 203;
        this.match(AmmParser.VARNAME);
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function ConstdeclarationContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_constdeclaration;
    return this;
}

ConstdeclarationContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
ConstdeclarationContext.prototype.constructor = ConstdeclarationContext;

ConstdeclarationContext.prototype.CONST = function() {
    return this.getToken(AmmParser.CONST, 0);
};

ConstdeclarationContext.prototype.decname = function() {
    return this.getTypedRuleContext(DecnameContext,0);
};

ConstdeclarationContext.prototype.TYPESEP = function() {
    return this.getToken(AmmParser.TYPESEP, 0);
};

ConstdeclarationContext.prototype.fulltypename = function() {
    return this.getTypedRuleContext(FulltypenameContext,0);
};

ConstdeclarationContext.prototype.EQUALS = function() {
    return this.getToken(AmmParser.EQUALS, 0);
};

ConstdeclarationContext.prototype.assignables = function() {
    return this.getTypedRuleContext(AssignablesContext,0);
};

ConstdeclarationContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

ConstdeclarationContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterConstdeclaration(this);
	}
};

ConstdeclarationContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitConstdeclaration(this);
	}
};




AmmParser.ConstdeclarationContext = ConstdeclarationContext;

AmmParser.prototype.constdeclaration = function() {

    var localctx = new ConstdeclarationContext(this, this._ctx, this.state);
    this.enterRule(localctx, 20, AmmParser.RULE_constdeclaration);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 205;
        this.match(AmmParser.CONST);
        this.state = 209;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 206;
            this.blank();
            this.state = 211;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 212;
        this.decname();
        this.state = 216;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 213;
            this.blank();
            this.state = 218;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 219;
        this.match(AmmParser.TYPESEP);
        this.state = 223;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 220;
            this.blank();
            this.state = 225;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 226;
        this.fulltypename();
        this.state = 230;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 227;
            this.blank();
            this.state = 232;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 233;
        this.match(AmmParser.EQUALS);
        this.state = 237;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 234;
            this.blank();
            this.state = 239;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 240;
        this.assignables();
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function LetdeclarationContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_letdeclaration;
    return this;
}

LetdeclarationContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
LetdeclarationContext.prototype.constructor = LetdeclarationContext;

LetdeclarationContext.prototype.LET = function() {
    return this.getToken(AmmParser.LET, 0);
};

LetdeclarationContext.prototype.decname = function() {
    return this.getTypedRuleContext(DecnameContext,0);
};

LetdeclarationContext.prototype.TYPESEP = function() {
    return this.getToken(AmmParser.TYPESEP, 0);
};

LetdeclarationContext.prototype.fulltypename = function() {
    return this.getTypedRuleContext(FulltypenameContext,0);
};

LetdeclarationContext.prototype.EQUALS = function() {
    return this.getToken(AmmParser.EQUALS, 0);
};

LetdeclarationContext.prototype.assignables = function() {
    return this.getTypedRuleContext(AssignablesContext,0);
};

LetdeclarationContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

LetdeclarationContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterLetdeclaration(this);
	}
};

LetdeclarationContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitLetdeclaration(this);
	}
};




AmmParser.LetdeclarationContext = LetdeclarationContext;

AmmParser.prototype.letdeclaration = function() {

    var localctx = new LetdeclarationContext(this, this._ctx, this.state);
    this.enterRule(localctx, 22, AmmParser.RULE_letdeclaration);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 242;
        this.match(AmmParser.LET);
        this.state = 246;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 243;
            this.blank();
            this.state = 248;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 249;
        this.decname();
        this.state = 253;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 250;
            this.blank();
            this.state = 255;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 256;
        this.match(AmmParser.TYPESEP);
        this.state = 260;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 257;
            this.blank();
            this.state = 262;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 263;
        this.fulltypename();
        this.state = 267;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 264;
            this.blank();
            this.state = 269;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 270;
        this.match(AmmParser.EQUALS);
        this.state = 274;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 271;
            this.blank();
            this.state = 276;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 277;
        this.assignables();
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function AssignmentsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_assignments;
    return this;
}

AssignmentsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
AssignmentsContext.prototype.constructor = AssignmentsContext;

AssignmentsContext.prototype.decname = function() {
    return this.getTypedRuleContext(DecnameContext,0);
};

AssignmentsContext.prototype.EQUALS = function() {
    return this.getToken(AmmParser.EQUALS, 0);
};

AssignmentsContext.prototype.assignables = function() {
    return this.getTypedRuleContext(AssignablesContext,0);
};

AssignmentsContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

AssignmentsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterAssignments(this);
	}
};

AssignmentsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitAssignments(this);
	}
};




AmmParser.AssignmentsContext = AssignmentsContext;

AmmParser.prototype.assignments = function() {

    var localctx = new AssignmentsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 24, AmmParser.RULE_assignments);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 279;
        this.decname();
        this.state = 283;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 280;
            this.blank();
            this.state = 285;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 286;
        this.match(AmmParser.EQUALS);
        this.state = 290;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 287;
            this.blank();
            this.state = 292;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 293;
        this.assignables();
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function AssignablesContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_assignables;
    return this;
}

AssignablesContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
AssignablesContext.prototype.constructor = AssignablesContext;

AssignablesContext.prototype.functions = function() {
    return this.getTypedRuleContext(FunctionsContext,0);
};

AssignablesContext.prototype.calls = function() {
    return this.getTypedRuleContext(CallsContext,0);
};

AssignablesContext.prototype.constants = function() {
    return this.getTypedRuleContext(ConstantsContext,0);
};

AssignablesContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

AssignablesContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterAssignables(this);
	}
};

AssignablesContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitAssignables(this);
	}
};




AmmParser.AssignablesContext = AssignablesContext;

AmmParser.prototype.assignables = function() {

    var localctx = new AssignablesContext(this, this._ctx, this.state);
    this.enterRule(localctx, 26, AmmParser.RULE_assignables);
    try {
        this.state = 299;
        this._errHandler.sync(this);
        var la_ = this._interp.adaptivePredict(this._input,42,this._ctx);
        switch(la_) {
        case 1:
            this.enterOuterAlt(localctx, 1);
            this.state = 295;
            this.functions();
            break;

        case 2:
            this.enterOuterAlt(localctx, 2);
            this.state = 296;
            this.calls();
            break;

        case 3:
            this.enterOuterAlt(localctx, 3);
            this.state = 297;
            this.constants();
            break;

        case 4:
            this.enterOuterAlt(localctx, 4);
            this.state = 298;
            this.match(AmmParser.VARNAME);
            break;

        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function CalllistContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_calllist;
    return this;
}

CalllistContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
CalllistContext.prototype.constructor = CalllistContext;

CalllistContext.prototype.VARNAME = function(i) {
	if(i===undefined) {
		i = null;
	}
    if(i===null) {
        return this.getTokens(AmmParser.VARNAME);
    } else {
        return this.getToken(AmmParser.VARNAME, i);
    }
};


CalllistContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

CalllistContext.prototype.SEP = function(i) {
	if(i===undefined) {
		i = null;
	}
    if(i===null) {
        return this.getTokens(AmmParser.SEP);
    } else {
        return this.getToken(AmmParser.SEP, i);
    }
};


CalllistContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterCalllist(this);
	}
};

CalllistContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitCalllist(this);
	}
};




AmmParser.CalllistContext = CalllistContext;

AmmParser.prototype.calllist = function() {

    var localctx = new CalllistContext(this, this._ctx, this.state);
    this.enterRule(localctx, 28, AmmParser.RULE_calllist);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 304;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 301;
            this.blank();
            this.state = 306;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 307;
        this.match(AmmParser.VARNAME);
        this.state = 318;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.SEP) {
            this.state = 308;
            this.match(AmmParser.SEP);
            this.state = 312;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
            while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
                this.state = 309;
                this.blank();
                this.state = 314;
                this._errHandler.sync(this);
                _la = this._input.LA(1);
            }
            this.state = 315;
            this.match(AmmParser.VARNAME);
            this.state = 320;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 324;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 321;
            this.blank();
            this.state = 326;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function CallsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_calls;
    return this;
}

CallsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
CallsContext.prototype.constructor = CallsContext;

CallsContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

CallsContext.prototype.OPENARGS = function() {
    return this.getToken(AmmParser.OPENARGS, 0);
};

CallsContext.prototype.CLOSEARGS = function() {
    return this.getToken(AmmParser.CLOSEARGS, 0);
};

CallsContext.prototype.WS = function(i) {
	if(i===undefined) {
		i = null;
	}
    if(i===null) {
        return this.getTokens(AmmParser.WS);
    } else {
        return this.getToken(AmmParser.WS, i);
    }
};


CallsContext.prototype.calllist = function() {
    return this.getTypedRuleContext(CalllistContext,0);
};

CallsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterCalls(this);
	}
};

CallsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitCalls(this);
	}
};




AmmParser.CallsContext = CallsContext;

AmmParser.prototype.calls = function() {

    var localctx = new CallsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 30, AmmParser.RULE_calls);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 327;
        this.match(AmmParser.VARNAME);
        this.state = 331;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.WS) {
            this.state = 328;
            this.match(AmmParser.WS);
            this.state = 333;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 334;
        this.match(AmmParser.OPENARGS);
        this.state = 336;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        if((((_la) & ~0x1f) == 0 && ((1 << _la) & ((1 << AmmParser.NEWLINE) | (1 << AmmParser.WS) | (1 << AmmParser.VARNAME))) !== 0)) {
            this.state = 335;
            this.calllist();
        }

        this.state = 338;
        this.match(AmmParser.CLOSEARGS);
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function EmitsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_emits;
    return this;
}

EmitsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
EmitsContext.prototype.constructor = EmitsContext;

EmitsContext.prototype.EMIT = function() {
    return this.getToken(AmmParser.EMIT, 0);
};

EmitsContext.prototype.VARNAME = function(i) {
	if(i===undefined) {
		i = null;
	}
    if(i===null) {
        return this.getTokens(AmmParser.VARNAME);
    } else {
        return this.getToken(AmmParser.VARNAME, i);
    }
};


EmitsContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

EmitsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterEmits(this);
	}
};

EmitsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitEmits(this);
	}
};




AmmParser.EmitsContext = EmitsContext;

AmmParser.prototype.emits = function() {

    var localctx = new EmitsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 32, AmmParser.RULE_emits);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 340;
        this.match(AmmParser.EMIT);
        this.state = 344;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 341;
            this.blank();
            this.state = 346;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 347;
        this.match(AmmParser.VARNAME);
        this.state = 355;
        this._errHandler.sync(this);
        var la_ = this._interp.adaptivePredict(this._input,51,this._ctx);
        if(la_===1) {
            this.state = 351;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
            while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
                this.state = 348;
                this.blank();
                this.state = 353;
                this._errHandler.sync(this);
                _la = this._input.LA(1);
            }
            this.state = 354;
            this.match(AmmParser.VARNAME);

        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function ConstantsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_constants;
    return this;
}

ConstantsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
ConstantsContext.prototype.constructor = ConstantsContext;

ConstantsContext.prototype.NUMBERCONSTANT = function() {
    return this.getToken(AmmParser.NUMBERCONSTANT, 0);
};

ConstantsContext.prototype.STRINGCONSTANT = function() {
    return this.getToken(AmmParser.STRINGCONSTANT, 0);
};

ConstantsContext.prototype.BOOLCONSTANT = function() {
    return this.getToken(AmmParser.BOOLCONSTANT, 0);
};

ConstantsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterConstants(this);
	}
};

ConstantsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitConstants(this);
	}
};




AmmParser.ConstantsContext = ConstantsContext;

AmmParser.prototype.constants = function() {

    var localctx = new ConstantsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 34, AmmParser.RULE_constants);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 357;
        _la = this._input.LA(1);
        if(!((((_la) & ~0x1f) == 0 && ((1 << _la) & ((1 << AmmParser.BOOLCONSTANT) | (1 << AmmParser.STRINGCONSTANT) | (1 << AmmParser.NUMBERCONSTANT))) !== 0))) {
        this._errHandler.recoverInline(this);
        }
        else {
        	this._errHandler.reportMatch(this);
            this.consume();
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function EventsContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_events;
    return this;
}

EventsContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
EventsContext.prototype.constructor = EventsContext;

EventsContext.prototype.EVENT = function() {
    return this.getToken(AmmParser.EVENT, 0);
};

EventsContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

EventsContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

EventsContext.prototype.TYPESEP = function() {
    return this.getToken(AmmParser.TYPESEP, 0);
};

EventsContext.prototype.typename = function() {
    return this.getTypedRuleContext(TypenameContext,0);
};

EventsContext.prototype.VOID = function() {
    return this.getToken(AmmParser.VOID, 0);
};

EventsContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterEvents(this);
	}
};

EventsContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitEvents(this);
	}
};




AmmParser.EventsContext = EventsContext;

AmmParser.prototype.events = function() {

    var localctx = new EventsContext(this, this._ctx, this.state);
    this.enterRule(localctx, 36, AmmParser.RULE_events);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 359;
        this.match(AmmParser.EVENT);
        this.state = 360;
        this.blank();
        this.state = 361;
        this.match(AmmParser.VARNAME);
        this.state = 365;
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        while(_la===AmmParser.NEWLINE || _la===AmmParser.WS) {
            this.state = 362;
            this.blank();
            this.state = 367;
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        }
        this.state = 368;
        this.match(AmmParser.TYPESEP);
        this.state = 371;
        this._errHandler.sync(this);
        switch(this._input.LA(1)) {
        case AmmParser.VARNAME:
            this.state = 369;
            this.typename();
            break;
        case AmmParser.VOID:
            this.state = 370;
            this.match(AmmParser.VOID);
            break;
        default:
            throw new antlr4.error.NoViableAltException(this);
        }
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


function HandlersContext(parser, parent, invokingState) {
	if(parent===undefined) {
	    parent = null;
	}
	if(invokingState===undefined || invokingState===null) {
		invokingState = -1;
	}
	antlr4.ParserRuleContext.call(this, parent, invokingState);
    this.parser = parser;
    this.ruleIndex = AmmParser.RULE_handlers;
    return this;
}

HandlersContext.prototype = Object.create(antlr4.ParserRuleContext.prototype);
HandlersContext.prototype.constructor = HandlersContext;

HandlersContext.prototype.ON = function() {
    return this.getToken(AmmParser.ON, 0);
};

HandlersContext.prototype.VARNAME = function() {
    return this.getToken(AmmParser.VARNAME, 0);
};

HandlersContext.prototype.functions = function() {
    return this.getTypedRuleContext(FunctionsContext,0);
};

HandlersContext.prototype.blank = function(i) {
    if(i===undefined) {
        i = null;
    }
    if(i===null) {
        return this.getTypedRuleContexts(BlankContext);
    } else {
        return this.getTypedRuleContext(BlankContext,i);
    }
};

HandlersContext.prototype.enterRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.enterHandlers(this);
	}
};

HandlersContext.prototype.exitRule = function(listener) {
    if(listener instanceof AmmListener ) {
        listener.exitHandlers(this);
	}
};




AmmParser.HandlersContext = HandlersContext;

AmmParser.prototype.handlers = function() {

    var localctx = new HandlersContext(this, this._ctx, this.state);
    this.enterRule(localctx, 38, AmmParser.RULE_handlers);
    var _la = 0; // Token type
    try {
        this.enterOuterAlt(localctx, 1);
        this.state = 373;
        this.match(AmmParser.ON);
        this.state = 375; 
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        do {
            this.state = 374;
            this.blank();
            this.state = 377; 
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        } while(_la===AmmParser.NEWLINE || _la===AmmParser.WS);
        this.state = 379;
        this.match(AmmParser.VARNAME);
        this.state = 381; 
        this._errHandler.sync(this);
        _la = this._input.LA(1);
        do {
            this.state = 380;
            this.blank();
            this.state = 383; 
            this._errHandler.sync(this);
            _la = this._input.LA(1);
        } while(_la===AmmParser.NEWLINE || _la===AmmParser.WS);
        this.state = 385;
        this.functions();
    } catch (re) {
    	if(re instanceof antlr4.error.RecognitionException) {
	        localctx.exception = re;
	        this._errHandler.reportError(this, re);
	        this._errHandler.recover(this, re);
	    } else {
	    	throw re;
	    }
    } finally {
        this.exitRule();
    }
    return localctx;
};


exports.AmmParser = AmmParser;
