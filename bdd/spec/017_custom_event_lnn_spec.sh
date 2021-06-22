Include build_tools.sh

Describe "Custom events"
  Describe "loop custom event"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        event loop: int64

        on loop fn looper(val: int64) {
          print(val);
          if val >= 10 {
            emit exit 0;
          } else {
            emit loop val + 1 || 0;
          }
        }

        on start {
          emit loop 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    OUTPUT="0
1
2
3
4
5
6
7
8
9
10"

    It "runs js"
      Pending generics-and-arithmetic
      When run test_js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      Pending generics-and-arithmetic
      When run test_agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "event with user-defined type"
    before() {
      lnn_sourceToAll "
        from @std/app import start, print, exit

        type Thing {
          foo: int64,
          bar: string
        }

        event thing: Thing

        on thing fn (t: Thing) {
          print(t.foo);
          print(t.bar);
          emit exit 0;
        }

        on start {
          emit thing new Thing {
            foo: 1,
            bar: 'baz'
          };
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    THINGOUTPUT="1
baz"

    It "runs js"
      When run test_js
      The output should eq "$THINGOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$THINGOUTPUT"
    End
  End

  Describe "multiple event handlers for an event"
    before() {
      lnn_sourceToAll "
        from @std/app import start, print, exit

        event aString: string

        on aString fn(str: string) {
          print('hey I got a string! ' + str);
        }

        on aString fn(str: string) {
          print('I also got a string! ' + str);
        }

        on aString fn(ignore: string) {
          wait(100);
          emit exit 0;
        }

        on start {
          emit aString 'hi';
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    THINGOUTPUT1="hey I got a string! hi"
    THINGOUTPUT2="I also got a string! hi"

    It "runs js"
      When run test_js
      The output should include "$THINGOUTPUT1"
      The output should include "$THINGOUTPUT2"
    End

    It "runs agc"
      When run test_agc
      The output should include "$THINGOUTPUT1"
      The output should include "$THINGOUTPUT2"
    End
  End
End
