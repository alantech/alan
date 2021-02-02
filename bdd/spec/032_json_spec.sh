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

    BASICOUTPUT="1.0
true
\"Hello, JSON!\"
[1.0, 2.0, 5.0]
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
End
