// Generated from Ln.g4 by ANTLR 4.7.2
// jshint ignore: start
var antlr4 = require('antlr4/index');



var serializedATN = ["\u0003\u608b\ua72a\u8133\ub9ed\u417c\u3be7\u7786\u5964",
    "\u0002/\u015c\b\u0001\u0004\u0002\t\u0002\u0004\u0003\t\u0003\u0004",
    "\u0004\t\u0004\u0004\u0005\t\u0005\u0004\u0006\t\u0006\u0004\u0007\t",
    "\u0007\u0004\b\t\b\u0004\t\t\t\u0004\n\t\n\u0004\u000b\t\u000b\u0004",
    "\f\t\f\u0004\r\t\r\u0004\u000e\t\u000e\u0004\u000f\t\u000f\u0004\u0010",
    "\t\u0010\u0004\u0011\t\u0011\u0004\u0012\t\u0012\u0004\u0013\t\u0013",
    "\u0004\u0014\t\u0014\u0004\u0015\t\u0015\u0004\u0016\t\u0016\u0004\u0017",
    "\t\u0017\u0004\u0018\t\u0018\u0004\u0019\t\u0019\u0004\u001a\t\u001a",
    "\u0004\u001b\t\u001b\u0004\u001c\t\u001c\u0004\u001d\t\u001d\u0004\u001e",
    "\t\u001e\u0004\u001f\t\u001f\u0004 \t \u0004!\t!\u0004\"\t\"\u0004#",
    "\t#\u0004$\t$\u0004%\t%\u0004&\t&\u0004\'\t\'\u0004(\t(\u0004)\t)\u0004",
    "*\t*\u0004+\t+\u0004,\t,\u0004-\t-\u0004.\t.\u0003\u0002\u0003\u0002",
    "\u0003\u0002\u0003\u0002\u0003\u0002\u0003\u0002\u0003\u0002\u0003\u0003",
    "\u0003\u0003\u0003\u0003\u0003\u0003\u0003\u0003\u0003\u0004\u0003\u0004",
    "\u0003\u0004\u0003\u0004\u0003\u0004\u0003\u0005\u0003\u0005\u0003\u0005",
    "\u0003\u0006\u0003\u0006\u0003\u0006\u0003\u0006\u0003\u0006\u0003\u0006",
    "\u0003\u0007\u0003\u0007\u0003\u0007\u0003\b\u0003\b\u0003\b\u0003\b",
    "\u0003\b\u0003\b\u0003\b\u0003\t\u0003\t\u0003\t\u0003\t\u0003\t\u0003",
    "\t\u0003\n\u0003\n\u0003\n\u0003\n\u0003\u000b\u0003\u000b\u0003\u000b",
    "\u0003\u000b\u0003\u000b\u0003\u000b\u0003\u000b\u0003\f\u0003\f\u0003",
    "\f\u0003\f\u0003\f\u0003\r\u0003\r\u0003\r\u0003\u000e\u0003\u000e\u0003",
    "\u000e\u0003\u000e\u0003\u000e\u0003\u000e\u0003\u000e\u0003\u000e\u0003",
    "\u000e\u0005\u000e\u00a4\n\u000e\u0003\u000f\u0003\u000f\u0003\u000f",
    "\u0003\u000f\u0003\u000f\u0003\u000f\u0003\u000f\u0003\u0010\u0003\u0010",
    "\u0003\u0010\u0003\u0010\u0003\u0010\u0003\u0010\u0003\u0011\u0003\u0011",
    "\u0003\u0011\u0003\u0011\u0003\u0011\u0003\u0011\u0003\u0011\u0003\u0011",
    "\u0003\u0011\u0003\u0011\u0003\u0011\u0003\u0012\u0003\u0012\u0003\u0012",
    "\u0003\u0013\u0003\u0013\u0003\u0013\u0003\u0013\u0003\u0013\u0003\u0014",
    "\u0003\u0014\u0003\u0014\u0003\u0014\u0003\u0015\u0003\u0015\u0003\u0015",
    "\u0003\u0015\u0003\u0015\u0003\u0015\u0003\u0015\u0003\u0015\u0003\u0015",
    "\u0003\u0015\u0003\u0016\u0003\u0016\u0007\u0016\u00d6\n\u0016\f\u0016",
    "\u000e\u0016\u00d9\u000b\u0016\u0003\u0017\u0003\u0017\u0003\u0018\u0003",
    "\u0018\u0003\u0019\u0003\u0019\u0003\u001a\u0003\u001a\u0003\u001b\u0003",
    "\u001b\u0003\u001c\u0003\u001c\u0003\u001d\u0003\u001d\u0003\u001e\u0003",
    "\u001e\u0003\u001f\u0003\u001f\u0003 \u0003 \u0003!\u0003!\u0003\"\u0003",
    "\"\u0003\"\u0003#\u0003#\u0003#\u0003#\u0003$\u0003$\u0003%\u0003%\u0003",
    "&\u0003&\u0003\'\u0003\'\u0003\'\u0005\'\u0101\n\'\u0003(\u0006(\u0104",
    "\n(\r(\u000e(\u0105\u0003)\u0003)\u0003)\u0003)\u0006)\u010c\n)\r)\u000e",
    ")\u010d\u0003)\u0003)\u0003*\u0003*\u0003*\u0003*\u0003*\u0003*\u0007",
    "*\u0118\n*\f*\u000e*\u011b\u000b*\u0003*\u0003*\u0003*\u0003*\u0003",
    "*\u0003+\u0003+\u0007+\u0124\n+\f+\u000e+\u0127\u000b+\u0003+\u0003",
    "+\u0003+\u0007+\u012c\n+\f+\u000e+\u012f\u000b+\u0003+\u0005+\u0132",
    "\n+\u0003,\u0003,\u0003,\u0003,\u0006,\u0138\n,\r,\u000e,\u0139\u0003",
    ",\u0006,\u013d\n,\r,\u000e,\u013e\u0003,\u0003,\u0006,\u0143\n,\r,\u000e",
    ",\u0144\u0005,\u0147\n,\u0005,\u0149\n,\u0003-\u0003-\u0007-\u014d\n",
    "-\f-\u000e-\u0150\u000b-\u0003.\u0006.\u0153\n.\r.\u000e.\u0154\u0003",
    ".\u0007.\u0158\n.\f.\u000e.\u015b\u000b.\u0002\u0002/\u0003\u0003\u0005",
    "\u0004\u0007\u0005\t\u0006\u000b\u0007\r\b\u000f\t\u0011\n\u0013\u000b",
    "\u0015\f\u0017\r\u0019\u000e\u001b\u000f\u001d\u0010\u001f\u0011!\u0012",
    "#\u0013%\u0014\'\u0015)\u0016+\u0017-\u0018/\u00191\u001a3\u001b5\u001c",
    "7\u001d9\u001e;\u001f= ?!A\"C#E$G%I&K\'M(O)Q*S+U,W-Y.[/\u0003\u0002",
    "\u000f\u0004\u0002\f\f\u000f\u000f\u0004\u0002\u000b\u000b\"\"\u0003",
    "\u000211\u0003\u0002,,\u0003\u0002$$\u0003\u0002))\u0005\u00022;CHc",
    "h\u0003\u00022;\u0003\u000200\f\u0002##%(,-/1<?AB``bb~~\u0080\u0080",
    "\u000b\u0002##%(,-/1<B``bb~~\u0080\u0080\u0005\u0002C\\aac|\u0006\u0002",
    "2;C\\aac|\u0002\u016d\u0002\u0003\u0003\u0002\u0002\u0002\u0002\u0005",
    "\u0003\u0002\u0002\u0002\u0002\u0007\u0003\u0002\u0002\u0002\u0002\t",
    "\u0003\u0002\u0002\u0002\u0002\u000b\u0003\u0002\u0002\u0002\u0002\r",
    "\u0003\u0002\u0002\u0002\u0002\u000f\u0003\u0002\u0002\u0002\u0002\u0011",
    "\u0003\u0002\u0002\u0002\u0002\u0013\u0003\u0002\u0002\u0002\u0002\u0015",
    "\u0003\u0002\u0002\u0002\u0002\u0017\u0003\u0002\u0002\u0002\u0002\u0019",
    "\u0003\u0002\u0002\u0002\u0002\u001b\u0003\u0002\u0002\u0002\u0002\u001d",
    "\u0003\u0002\u0002\u0002\u0002\u001f\u0003\u0002\u0002\u0002\u0002!",
    "\u0003\u0002\u0002\u0002\u0002#\u0003\u0002\u0002\u0002\u0002%\u0003",
    "\u0002\u0002\u0002\u0002\'\u0003\u0002\u0002\u0002\u0002)\u0003\u0002",
    "\u0002\u0002\u0002+\u0003\u0002\u0002\u0002\u0002-\u0003\u0002\u0002",
    "\u0002\u0002/\u0003\u0002\u0002\u0002\u00021\u0003\u0002\u0002\u0002",
    "\u00023\u0003\u0002\u0002\u0002\u00025\u0003\u0002\u0002\u0002\u0002",
    "7\u0003\u0002\u0002\u0002\u00029\u0003\u0002\u0002\u0002\u0002;\u0003",
    "\u0002\u0002\u0002\u0002=\u0003\u0002\u0002\u0002\u0002?\u0003\u0002",
    "\u0002\u0002\u0002A\u0003\u0002\u0002\u0002\u0002C\u0003\u0002\u0002",
    "\u0002\u0002E\u0003\u0002\u0002\u0002\u0002G\u0003\u0002\u0002\u0002",
    "\u0002I\u0003\u0002\u0002\u0002\u0002K\u0003\u0002\u0002\u0002\u0002",
    "M\u0003\u0002\u0002\u0002\u0002O\u0003\u0002\u0002\u0002\u0002Q\u0003",
    "\u0002\u0002\u0002\u0002S\u0003\u0002\u0002\u0002\u0002U\u0003\u0002",
    "\u0002\u0002\u0002W\u0003\u0002\u0002\u0002\u0002Y\u0003\u0002\u0002",
    "\u0002\u0002[\u0003\u0002\u0002\u0002\u0003]\u0003\u0002\u0002\u0002",
    "\u0005d\u0003\u0002\u0002\u0002\u0007i\u0003\u0002\u0002\u0002\tn\u0003",
    "\u0002\u0002\u0002\u000bq\u0003\u0002\u0002\u0002\rw\u0003\u0002\u0002",
    "\u0002\u000fz\u0003\u0002\u0002\u0002\u0011\u0081\u0003\u0002\u0002",
    "\u0002\u0013\u0087\u0003\u0002\u0002\u0002\u0015\u008b\u0003\u0002\u0002",
    "\u0002\u0017\u0092\u0003\u0002\u0002\u0002\u0019\u0097\u0003\u0002\u0002",
    "\u0002\u001b\u00a3\u0003\u0002\u0002\u0002\u001d\u00a5\u0003\u0002\u0002",
    "\u0002\u001f\u00ac\u0003\u0002\u0002\u0002!\u00b2\u0003\u0002\u0002",
    "\u0002#\u00bd\u0003\u0002\u0002\u0002%\u00c0\u0003\u0002\u0002\u0002",
    "\'\u00c5\u0003\u0002\u0002\u0002)\u00c9\u0003\u0002\u0002\u0002+\u00d3",
    "\u0003\u0002\u0002\u0002-\u00da\u0003\u0002\u0002\u0002/\u00dc\u0003",
    "\u0002\u0002\u00021\u00de\u0003\u0002\u0002\u00023\u00e0\u0003\u0002",
    "\u0002\u00025\u00e2\u0003\u0002\u0002\u00027\u00e4\u0003\u0002\u0002",
    "\u00029\u00e6\u0003\u0002\u0002\u0002;\u00e8\u0003\u0002\u0002\u0002",
    "=\u00ea\u0003\u0002\u0002\u0002?\u00ec\u0003\u0002\u0002\u0002A\u00ee",
    "\u0003\u0002\u0002\u0002C\u00f0\u0003\u0002\u0002\u0002E\u00f3\u0003",
    "\u0002\u0002\u0002G\u00f7\u0003\u0002\u0002\u0002I\u00f9\u0003\u0002",
    "\u0002\u0002K\u00fb\u0003\u0002\u0002\u0002M\u0100\u0003\u0002\u0002",
    "\u0002O\u0103\u0003\u0002\u0002\u0002Q\u0107\u0003\u0002\u0002\u0002",
    "S\u0111\u0003\u0002\u0002\u0002U\u0131\u0003\u0002\u0002\u0002W\u0148",
    "\u0003\u0002\u0002\u0002Y\u014a\u0003\u0002\u0002\u0002[\u0152\u0003",
    "\u0002\u0002\u0002]^\u0007k\u0002\u0002^_\u0007o\u0002\u0002_`\u0007",
    "r\u0002\u0002`a\u0007q\u0002\u0002ab\u0007t\u0002\u0002bc\u0007v\u0002",
    "\u0002c\u0004\u0003\u0002\u0002\u0002de\u0007h\u0002\u0002ef\u0007t",
    "\u0002\u0002fg\u0007q\u0002\u0002gh\u0007o\u0002\u0002h\u0006\u0003",
    "\u0002\u0002\u0002ij\u0007v\u0002\u0002jk\u0007{\u0002\u0002kl\u0007",
    "r\u0002\u0002lm\u0007g\u0002\u0002m\b\u0003\u0002\u0002\u0002no\u0007",
    "h\u0002\u0002op\u0007p\u0002\u0002p\n\u0003\u0002\u0002\u0002qr\u0007",
    "g\u0002\u0002rs\u0007x\u0002\u0002st\u0007g\u0002\u0002tu\u0007p\u0002",
    "\u0002uv\u0007v\u0002\u0002v\f\u0003\u0002\u0002\u0002wx\u0007q\u0002",
    "\u0002xy\u0007p\u0002\u0002y\u000e\u0003\u0002\u0002\u0002z{\u0007g",
    "\u0002\u0002{|\u0007z\u0002\u0002|}\u0007r\u0002\u0002}~\u0007q\u0002",
    "\u0002~\u007f\u0007t\u0002\u0002\u007f\u0080\u0007v\u0002\u0002\u0080",
    "\u0010\u0003\u0002\u0002\u0002\u0081\u0082\u0007e\u0002\u0002\u0082",
    "\u0083\u0007q\u0002\u0002\u0083\u0084\u0007p\u0002\u0002\u0084\u0085",
    "\u0007u\u0002\u0002\u0085\u0086\u0007v\u0002\u0002\u0086\u0012\u0003",
    "\u0002\u0002\u0002\u0087\u0088\u0007n\u0002\u0002\u0088\u0089\u0007",
    "g\u0002\u0002\u0089\u008a\u0007v\u0002\u0002\u008a\u0014\u0003\u0002",
    "\u0002\u0002\u008b\u008c\u0007t\u0002\u0002\u008c\u008d\u0007g\u0002",
    "\u0002\u008d\u008e\u0007v\u0002\u0002\u008e\u008f\u0007w\u0002\u0002",
    "\u008f\u0090\u0007t\u0002\u0002\u0090\u0091\u0007p\u0002\u0002\u0091",
    "\u0016\u0003\u0002\u0002\u0002\u0092\u0093\u0007g\u0002\u0002\u0093",
    "\u0094\u0007o\u0002\u0002\u0094\u0095\u0007k\u0002\u0002\u0095\u0096",
    "\u0007v\u0002\u0002\u0096\u0018\u0003\u0002\u0002\u0002\u0097\u0098",
    "\u0007c\u0002\u0002\u0098\u0099\u0007u\u0002\u0002\u0099\u001a\u0003",
    "\u0002\u0002\u0002\u009a\u009b\u0007v\u0002\u0002\u009b\u009c\u0007",
    "t\u0002\u0002\u009c\u009d\u0007w\u0002\u0002\u009d\u00a4\u0007g\u0002",
    "\u0002\u009e\u009f\u0007h\u0002\u0002\u009f\u00a0\u0007c\u0002\u0002",
    "\u00a0\u00a1\u0007n\u0002\u0002\u00a1\u00a2\u0007u\u0002\u0002\u00a2",
    "\u00a4\u0007g\u0002\u0002\u00a3\u009a\u0003\u0002\u0002\u0002\u00a3",
    "\u009e\u0003\u0002\u0002\u0002\u00a4\u001c\u0003\u0002\u0002\u0002\u00a5",
    "\u00a6\u0007r\u0002\u0002\u00a6\u00a7\u0007t\u0002\u0002\u00a7\u00a8",
    "\u0007g\u0002\u0002\u00a8\u00a9\u0007h\u0002\u0002\u00a9\u00aa\u0007",
    "k\u0002\u0002\u00aa\u00ab\u0007z\u0002\u0002\u00ab\u001e\u0003\u0002",
    "\u0002\u0002\u00ac\u00ad\u0007k\u0002\u0002\u00ad\u00ae\u0007p\u0002",
    "\u0002\u00ae\u00af\u0007h\u0002\u0002\u00af\u00b0\u0007k\u0002\u0002",
    "\u00b0\u00b1\u0007z\u0002\u0002\u00b1 \u0003\u0002\u0002\u0002\u00b2",
    "\u00b3\u0007r\u0002\u0002\u00b3\u00b4\u0007t\u0002\u0002\u00b4\u00b5",
    "\u0007g\u0002\u0002\u00b5\u00b6\u0007e\u0002\u0002\u00b6\u00b7\u0007",
    "g\u0002\u0002\u00b7\u00b8\u0007f\u0002\u0002\u00b8\u00b9\u0007g\u0002",
    "\u0002\u00b9\u00ba\u0007p\u0002\u0002\u00ba\u00bb\u0007e\u0002\u0002",
    "\u00bb\u00bc\u0007g\u0002\u0002\u00bc\"\u0003\u0002\u0002\u0002\u00bd",
    "\u00be\u0007k\u0002\u0002\u00be\u00bf\u0007h\u0002\u0002\u00bf$\u0003",
    "\u0002\u0002\u0002\u00c0\u00c1\u0007g\u0002\u0002\u00c1\u00c2\u0007",
    "n\u0002\u0002\u00c2\u00c3\u0007u\u0002\u0002\u00c3\u00c4\u0007g\u0002",
    "\u0002\u00c4&\u0003\u0002\u0002\u0002\u00c5\u00c6\u0007p\u0002\u0002",
    "\u00c6\u00c7\u0007g\u0002\u0002\u00c7\u00c8\u0007y\u0002\u0002\u00c8",
    "(\u0003\u0002\u0002\u0002\u00c9\u00ca\u0007k\u0002\u0002\u00ca\u00cb",
    "\u0007p\u0002\u0002\u00cb\u00cc\u0007v\u0002\u0002\u00cc\u00cd\u0007",
    "g\u0002\u0002\u00cd\u00ce\u0007t\u0002\u0002\u00ce\u00cf\u0007h\u0002",
    "\u0002\u00cf\u00d0\u0007c\u0002\u0002\u00d0\u00d1\u0007e\u0002\u0002",
    "\u00d1\u00d2\u0007g\u0002\u0002\u00d2*\u0003\u0002\u0002\u0002\u00d3",
    "\u00d7\u0007.\u0002\u0002\u00d4\u00d6\u0005O(\u0002\u00d5\u00d4\u0003",
    "\u0002\u0002\u0002\u00d6\u00d9\u0003\u0002\u0002\u0002\u00d7\u00d5\u0003",
    "\u0002\u0002\u0002\u00d7\u00d8\u0003\u0002\u0002\u0002\u00d8,\u0003",
    "\u0002\u0002\u0002\u00d9\u00d7\u0003\u0002\u0002\u0002\u00da\u00db\u0007",
    "}\u0002\u0002\u00db.\u0003\u0002\u0002\u0002\u00dc\u00dd\u0007\u007f",
    "\u0002\u0002\u00dd0\u0003\u0002\u0002\u0002\u00de\u00df\u0007*\u0002",
    "\u0002\u00df2\u0003\u0002\u0002\u0002\u00e0\u00e1\u0007+\u0002\u0002",
    "\u00e14\u0003\u0002\u0002\u0002\u00e2\u00e3\u0007>\u0002\u0002\u00e3",
    "6\u0003\u0002\u0002\u0002\u00e4\u00e5\u0007@\u0002\u0002\u00e58\u0003",
    "\u0002\u0002\u0002\u00e6\u00e7\u0007]\u0002\u0002\u00e7:\u0003\u0002",
    "\u0002\u0002\u00e8\u00e9\u0007_\u0002\u0002\u00e9<\u0003\u0002\u0002",
    "\u0002\u00ea\u00eb\u00070\u0002\u0002\u00eb>\u0003\u0002\u0002\u0002",
    "\u00ec\u00ed\u0007?\u0002\u0002\u00ed@\u0003\u0002\u0002\u0002\u00ee",
    "\u00ef\u0007B\u0002\u0002\u00efB\u0003\u0002\u0002\u0002\u00f0\u00f1",
    "\u00070\u0002\u0002\u00f1\u00f2\u00071\u0002\u0002\u00f2D\u0003\u0002",
    "\u0002\u0002\u00f3\u00f4\u00070\u0002\u0002\u00f4\u00f5\u00070\u0002",
    "\u0002\u00f5\u00f6\u00071\u0002\u0002\u00f6F\u0003\u0002\u0002\u0002",
    "\u00f7\u00f8\u00071\u0002\u0002\u00f8H\u0003\u0002\u0002\u0002\u00f9",
    "\u00fa\u0007~\u0002\u0002\u00faJ\u0003\u0002\u0002\u0002\u00fb\u00fc",
    "\u0007<\u0002\u0002\u00fcL\u0003\u0002\u0002\u0002\u00fd\u0101\t\u0002",
    "\u0002\u0002\u00fe\u00ff\u0007\u000f\u0002\u0002\u00ff\u0101\u0007\f",
    "\u0002\u0002\u0100\u00fd\u0003\u0002\u0002\u0002\u0100\u00fe\u0003\u0002",
    "\u0002\u0002\u0101N\u0003\u0002\u0002\u0002\u0102\u0104\t\u0003\u0002",
    "\u0002\u0103\u0102\u0003\u0002\u0002\u0002\u0104\u0105\u0003\u0002\u0002",
    "\u0002\u0105\u0103\u0003\u0002\u0002\u0002\u0105\u0106\u0003\u0002\u0002",
    "\u0002\u0106P\u0003\u0002\u0002\u0002\u0107\u0108\u00071\u0002\u0002",
    "\u0108\u0109\u00071\u0002\u0002\u0109\u010b\u0003\u0002\u0002\u0002",
    "\u010a\u010c\n\u0002\u0002\u0002\u010b\u010a\u0003\u0002\u0002\u0002",
    "\u010c\u010d\u0003\u0002\u0002\u0002\u010d\u010b\u0003\u0002\u0002\u0002",
    "\u010d\u010e\u0003\u0002\u0002\u0002\u010e\u010f\u0003\u0002\u0002\u0002",
    "\u010f\u0110\b)\u0002\u0002\u0110R\u0003\u0002\u0002\u0002\u0111\u0112",
    "\u00071\u0002\u0002\u0112\u0113\u0007,\u0002\u0002\u0113\u0119\u0003",
    "\u0002\u0002\u0002\u0114\u0115\u0007,\u0002\u0002\u0115\u0118\n\u0004",
    "\u0002\u0002\u0116\u0118\n\u0005\u0002\u0002\u0117\u0114\u0003\u0002",
    "\u0002\u0002\u0117\u0116\u0003\u0002\u0002\u0002\u0118\u011b\u0003\u0002",
    "\u0002\u0002\u0119\u0117\u0003\u0002\u0002\u0002\u0119\u011a\u0003\u0002",
    "\u0002\u0002\u011a\u011c\u0003\u0002\u0002\u0002\u011b\u0119\u0003\u0002",
    "\u0002\u0002\u011c\u011d\u0007,\u0002\u0002\u011d\u011e\u00071\u0002",
    "\u0002\u011e\u011f\u0003\u0002\u0002\u0002\u011f\u0120\b*\u0002\u0002",
    "\u0120T\u0003\u0002\u0002\u0002\u0121\u0125\u0007$\u0002\u0002\u0122",
    "\u0124\n\u0006\u0002\u0002\u0123\u0122\u0003\u0002\u0002\u0002\u0124",
    "\u0127\u0003\u0002\u0002\u0002\u0125\u0123\u0003\u0002\u0002\u0002\u0125",
    "\u0126\u0003\u0002\u0002\u0002\u0126\u0128\u0003\u0002\u0002\u0002\u0127",
    "\u0125\u0003\u0002\u0002\u0002\u0128\u0132\u0007$\u0002\u0002\u0129",
    "\u012d\u0007)\u0002\u0002\u012a\u012c\n\u0007\u0002\u0002\u012b\u012a",
    "\u0003\u0002\u0002\u0002\u012c\u012f\u0003\u0002\u0002\u0002\u012d\u012b",
    "\u0003\u0002\u0002\u0002\u012d\u012e\u0003\u0002\u0002\u0002\u012e\u0130",
    "\u0003\u0002\u0002\u0002\u012f\u012d\u0003\u0002\u0002\u0002\u0130\u0132",
    "\u0007)\u0002\u0002\u0131\u0121\u0003\u0002\u0002\u0002\u0131\u0129",
    "\u0003\u0002\u0002\u0002\u0132V\u0003\u0002\u0002\u0002\u0133\u0134",
    "\u00072\u0002\u0002\u0134\u0135\u0007z\u0002\u0002\u0135\u0137\u0003",
    "\u0002\u0002\u0002\u0136\u0138\t\b\u0002\u0002\u0137\u0136\u0003\u0002",
    "\u0002\u0002\u0138\u0139\u0003\u0002\u0002\u0002\u0139\u0137\u0003\u0002",
    "\u0002\u0002\u0139\u013a\u0003\u0002\u0002\u0002\u013a\u0149\u0003\u0002",
    "\u0002\u0002\u013b\u013d\t\t\u0002\u0002\u013c\u013b\u0003\u0002\u0002",
    "\u0002\u013d\u013e\u0003\u0002\u0002\u0002\u013e\u013c\u0003\u0002\u0002",
    "\u0002\u013e\u013f\u0003\u0002\u0002\u0002\u013f\u0146\u0003\u0002\u0002",
    "\u0002\u0140\u0142\t\n\u0002\u0002\u0141\u0143\t\t\u0002\u0002\u0142",
    "\u0141\u0003\u0002\u0002\u0002\u0143\u0144\u0003\u0002\u0002\u0002\u0144",
    "\u0142\u0003\u0002\u0002\u0002\u0144\u0145\u0003\u0002\u0002\u0002\u0145",
    "\u0147\u0003\u0002\u0002\u0002\u0146\u0140\u0003\u0002\u0002\u0002\u0146",
    "\u0147\u0003\u0002\u0002\u0002\u0147\u0149\u0003\u0002\u0002\u0002\u0148",
    "\u0133\u0003\u0002\u0002\u0002\u0148\u013c\u0003\u0002\u0002\u0002\u0149",
    "X\u0003\u0002\u0002\u0002\u014a\u014e\t\u000b\u0002\u0002\u014b\u014d",
    "\t\f\u0002\u0002\u014c\u014b\u0003\u0002\u0002\u0002\u014d\u0150\u0003",
    "\u0002\u0002\u0002\u014e\u014c\u0003\u0002\u0002\u0002\u014e\u014f\u0003",
    "\u0002\u0002\u0002\u014fZ\u0003\u0002\u0002\u0002\u0150\u014e\u0003",
    "\u0002\u0002\u0002\u0151\u0153\t\r\u0002\u0002\u0152\u0151\u0003\u0002",
    "\u0002\u0002\u0153\u0154\u0003\u0002\u0002\u0002\u0154\u0152\u0003\u0002",
    "\u0002\u0002\u0154\u0155\u0003\u0002\u0002\u0002\u0155\u0159\u0003\u0002",
    "\u0002\u0002\u0156\u0158\t\u000e\u0002\u0002\u0157\u0156\u0003\u0002",
    "\u0002\u0002\u0158\u015b\u0003\u0002\u0002\u0002\u0159\u0157\u0003\u0002",
    "\u0002\u0002\u0159\u015a\u0003\u0002\u0002\u0002\u015a\\\u0003\u0002",
    "\u0002\u0002\u015b\u0159\u0003\u0002\u0002\u0002\u0015\u0002\u00a3\u00d7",
    "\u0100\u0105\u010d\u0117\u0119\u0125\u012d\u0131\u0139\u013e\u0144\u0146",
    "\u0148\u014e\u0154\u0159\u0003\b\u0002\u0002"].join("");


var atn = new antlr4.atn.ATNDeserializer().deserialize(serializedATN);

var decisionsToDFA = atn.decisionToState.map( function(ds, index) { return new antlr4.dfa.DFA(ds, index); });

function LnLexer(input) {
	antlr4.Lexer.call(this, input);
    this._interp = new antlr4.atn.LexerATNSimulator(this, atn, decisionsToDFA, new antlr4.PredictionContextCache());
    return this;
}

LnLexer.prototype = Object.create(antlr4.Lexer.prototype);
LnLexer.prototype.constructor = LnLexer;

Object.defineProperty(LnLexer.prototype, "atn", {
        get : function() {
                return atn;
        }
});

LnLexer.EOF = antlr4.Token.EOF;
LnLexer.IMPORT = 1;
LnLexer.FROM = 2;
LnLexer.TYPE = 3;
LnLexer.FN = 4;
LnLexer.EVENT = 5;
LnLexer.ON = 6;
LnLexer.EXPORT = 7;
LnLexer.CONST = 8;
LnLexer.LET = 9;
LnLexer.RETURN = 10;
LnLexer.EMIT = 11;
LnLexer.AS = 12;
LnLexer.BOOLCONSTANT = 13;
LnLexer.PREFIX = 14;
LnLexer.INFIX = 15;
LnLexer.PRECEDENCE = 16;
LnLexer.IF = 17;
LnLexer.ELSE = 18;
LnLexer.NEW = 19;
LnLexer.INTERFACE = 20;
LnLexer.SEP = 21;
LnLexer.OPENBODY = 22;
LnLexer.CLOSEBODY = 23;
LnLexer.OPENARGS = 24;
LnLexer.CLOSEARGS = 25;
LnLexer.OPENGENERIC = 26;
LnLexer.CLOSEGENERIC = 27;
LnLexer.OPENARRAY = 28;
LnLexer.CLOSEARRAY = 29;
LnLexer.METHODSEP = 30;
LnLexer.EQUALS = 31;
LnLexer.GLOBAL = 32;
LnLexer.CURDIR = 33;
LnLexer.PARDIR = 34;
LnLexer.DIRSEP = 35;
LnLexer.OR = 36;
LnLexer.TYPESEP = 37;
LnLexer.NEWLINE = 38;
LnLexer.WS = 39;
LnLexer.SINGLELINECOMMENT = 40;
LnLexer.MULTILINECOMMENT = 41;
LnLexer.STRINGCONSTANT = 42;
LnLexer.NUMBERCONSTANT = 43;
LnLexer.GENERALOPERATORS = 44;
LnLexer.VARNAME = 45;

LnLexer.prototype.channelNames = [ "DEFAULT_TOKEN_CHANNEL", "HIDDEN" ];

LnLexer.prototype.modeNames = [ "DEFAULT_MODE" ];

LnLexer.prototype.literalNames = [ null, "'import'", "'from'", "'type'", 
                                   "'fn'", "'event'", "'on'", "'export'", 
                                   "'const'", "'let'", "'return'", "'emit'", 
                                   "'as'", null, "'prefix'", "'infix'", 
                                   "'precedence'", "'if'", "'else'", "'new'", 
                                   "'interface'", null, "'{'", "'}'", "'('", 
                                   "')'", "'<'", "'>'", "'['", "']'", "'.'", 
                                   "'='", "'@'", "'./'", "'../'", "'/'", 
                                   "'|'", "':'" ];

LnLexer.prototype.symbolicNames = [ null, "IMPORT", "FROM", "TYPE", "FN", 
                                    "EVENT", "ON", "EXPORT", "CONST", "LET", 
                                    "RETURN", "EMIT", "AS", "BOOLCONSTANT", 
                                    "PREFIX", "INFIX", "PRECEDENCE", "IF", 
                                    "ELSE", "NEW", "INTERFACE", "SEP", "OPENBODY", 
                                    "CLOSEBODY", "OPENARGS", "CLOSEARGS", 
                                    "OPENGENERIC", "CLOSEGENERIC", "OPENARRAY", 
                                    "CLOSEARRAY", "METHODSEP", "EQUALS", 
                                    "GLOBAL", "CURDIR", "PARDIR", "DIRSEP", 
                                    "OR", "TYPESEP", "NEWLINE", "WS", "SINGLELINECOMMENT", 
                                    "MULTILINECOMMENT", "STRINGCONSTANT", 
                                    "NUMBERCONSTANT", "GENERALOPERATORS", 
                                    "VARNAME" ];

LnLexer.prototype.ruleNames = [ "IMPORT", "FROM", "TYPE", "FN", "EVENT", 
                                "ON", "EXPORT", "CONST", "LET", "RETURN", 
                                "EMIT", "AS", "BOOLCONSTANT", "PREFIX", 
                                "INFIX", "PRECEDENCE", "IF", "ELSE", "NEW", 
                                "INTERFACE", "SEP", "OPENBODY", "CLOSEBODY", 
                                "OPENARGS", "CLOSEARGS", "OPENGENERIC", 
                                "CLOSEGENERIC", "OPENARRAY", "CLOSEARRAY", 
                                "METHODSEP", "EQUALS", "GLOBAL", "CURDIR", 
                                "PARDIR", "DIRSEP", "OR", "TYPESEP", "NEWLINE", 
                                "WS", "SINGLELINECOMMENT", "MULTILINECOMMENT", 
                                "STRINGCONSTANT", "NUMBERCONSTANT", "GENERALOPERATORS", 
                                "VARNAME" ];

LnLexer.prototype.grammarFileName = "Ln.g4";



exports.LnLexer = LnLexer;

