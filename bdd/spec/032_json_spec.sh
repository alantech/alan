Include build_tools.sh

Describe "JSON"
  Describe "basic construction and printing"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/json import JSON, toJSON, toString, JSONBase, JSONNode, IsObject, Null

        on start {
          1.0.toJSON().print();
          true.toJSON().print();
          'Hello, JSON!'.toJSON().print();
          [1.0, 2.0, 5.0].toJSON().print();
          toJSON().print();

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    BASICOUTPUT="1
true
\"Hello, JSON!\"
[1, 2, 5]
null"

    It "runs js"
      When run test_js
      The output should eq "$BASICOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$BASICOUTPUT"
    End
  End

  Describe "complex JSON type construction and printing"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/json import JSON, toString, JSONBase, JSONNode, IsObject, Null, newJSONObject, newJSONArray, addKeyVal, push

        on start {
          newJSONObject()
            .addKeyVal('mixed', 'values')
            .addKeyVal('work', true)
            .addKeyVal('even', newJSONArray()
              .push(4.0)
              .push('arrays'))
            .print();

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    COMPLEXOUTPUT="{\"mixed\": \"values\", \"work\": true, \"even\": [4, \"arrays\"]}"

    It "runs js"
      When run test_js
      The output should eq "$COMPLEXOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$COMPLEXOUTPUT"
    End
  End
  
  Describe "JSON parsing"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/json import JSON, toJSON, JSONBase, JSONNode, IsObject, Null, parse, isBool, getBool, toJSONNode, isNull

        on start {
          print('Parsing \"true\"');
          'true'.parse().getOr(toJSON()).getRootNode().getOr(toJSONNode()).isBool().print();
          'true'.parse().getOr(toJSON()).getRootNode().getOr(toJSONNode()).getBool().print();

          print('Parsing \"false\"');
          const falseJson = 'false'.parse().getOr(toJSON());
          falseJson.getRootNode().getOr(toJSONNode()).isBool().print();
          falseJson.getRootNode().getOr(toJSONNode()).getBool().print();

          print('Parsing \"null\"');
          const nullJson = 'null'.parse().getOr(toJSON(true));
          nullJson.getRootNode().getOr(toJSONNode(true)).isNull().print();

          print('Parsing \"garbage\"');
          'garbage'.parse().isOk().print();

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    PARSEOUTPUT="Parsing \"true\"
true
true
Parsing \"false\"
true
false
Parsing \"null\"
true
Parsing \"garbage\"
false"

    It "runs js"
      When run test_js
      The output should eq "$PARSEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$PARSEOUTPUT"
    End
  End
End
