Include build_tools.sh

Describe "Comparators"
  Describe "Equals"
    before() {
      # TODO: lnn_sourceToAll 
      sourceToTemp "
        from @std/app import start, stdout, exit

        on start {
          // constrained to an int8 from the emit exit at the bottom
          const i8val = 0;
          emit stdout toString(i8val == 0) + '\n';
          wait(1000);
          emit stdout toString(i8val == 1) + '\n';
          wait(1000);

          const i16val: int16 = 0;
          emit stdout toString(i16val == 0) + '\n';
          wait(1000);
          emit stdout toString(i16val == 1) + '\n';
          wait(1000);

          const i32val: int32 = 0;
          emit stdout toString(i32val == 0) + '\n';
          wait(1000);
          emit stdout toString(i32val == 1) + '\n';
          wait(1000);

          const i64val: int64 = 0;
          emit stdout toString(i64val == 0) + '\n';
          wait(1000);
          emit stdout toString(i64val == 1) + '\n';
          wait(1000);

          const f32val: float32 = 0.0;
          emit stdout toString(f32val == 0.0) + '\n';
          wait(1000);
          emit stdout toString(f32val == 1.0) + '\n';
          wait(1000);

          const f64val: float64 = 0.0;
          emit stdout toString(f64val == 0.0) + '\n';
          wait(1000);
          emit stdout toString(f64val == 1.0) + '\n';
          wait(1000);

          emit stdout toString(true == true)) + '\n';
          wait(1000);
          emit stdout toString(true == false) + '\n';
          wait(1000);

          emit stdout toString('hello' + \"hello\") + '\n';
          wait(1000);
          emit stdout toString('hello' + \"world\") + '\n';

          wait(1000);
          emit exit i8val;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EQUALS="true
false
true
false
true
false
true
false
true
false
true
false
true
false
true
false"

    It "runs js"
      Pending types-depending-on-each-other
      When run test_js
      The output should eq "$EQUALS"
    End

    It "runs agc"
      Pending types-depending-on-each-other
      When run test_agc
      The output should eq "$EQUALS"
    End
  End

  Describe "Not Equals"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start { 
          const i8val = 0;
          emit stdout toString(i8val != 0);
          wait(10);
          emit stdout toString(i8val != 1);
          wait(10);

          const i16val: int16 = 0;
          emit stdout toString(i16val != 0);
          wait(10);
          emit stdout toString(i16val != 1);
          wait(10);

          const i32val: int32 = 0;
          emit stdout toString(i32val != 0);
          wait(10);
          emit stdout toString(i32val != 1);
          wait(10);

          const i64val: int64 = 0;
          emit stdout toString(i64val != 0);
          wait(10);
          emit stdout toString(i64val != 1);
          wait(10);

          const f32val: float32 = 0;
          emit stdout toString(f32val != 0.0);
          wait(10);
          emit stdout toString(f32val != 1.0);
          wait(10);

          const f64val: float64 = 0;
          emit stdout toString(f64val != 0.0);
          wait(10);
          emit stdout toString(f64val != 1.0);
          wait(10);

          emit stdout toString(true != true);
          wait(10);
          emit stdout toString(true != false);
          wait(10);

          emit exit i8val;
       }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    NOTEQUALS="false
true
false
true
false
true
false
true
false
true
false
true
false
true"

    It "runs js"
      When run test_js
      The output should eq "$NOTEQUALS"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$NOTEQUALS"
    End
  End

  Describe "Less Than"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          const i8val = 0;
          emit stdout toString(i8val < 1);
          wait(10);
          emit stdout toString(1 < i8val);
          wait(10);

          const i16val: int16 = 0;
          emit stdout toString(i16val < 1);
          wait(10);
          emit stdout toString(1 < i16val);
          wait(10);

          const i32val: int32 = 0;
          emit stdout toString(i32val < 1);
          wait(10);
          emit stdout toString(1 < i32val);
          wait(10);

          const i64val: int64 = 0;
          emit stdout toString(i64val < 1);
          wait(10);
          emit stdout toString(1 < i64val);
          wait(10);

          emit stdout toString('hello' < 'world');
          wait(10);
          emit stdout toString('hello' < 'hello');
          wait(10);

          emit exit i8val;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    LESSTHAN="true
false
true
false
true
false
true
false
true
false
true
false
true
false"

    It "runs js"
      When run test_js
      The output should eq "$LESSTHAN"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$LESSTHAN"
    End
  End

  Describe "Less Than Or Equal"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          const i8val = 0;
          emit stdout toString(i8val <= 1);
          wait(10);
          emit stdout toString(1 <= i8val);
          wait(10);

          const i16val: int16 = 0;
          emit stdout toString(i16val <= 1);
          wait(10);
          emit stdout toString(1 <= i16val);
          wait(10);

          const i32val: int32 = 0;
          emit stdout toString(i32val <= 1);
          wait(10);
          emit stdout toString(1 <= i32val);
          wait(10);

          const i64val: int64 = 0;
          emit stdout toString(i64val <= 1);
          wait(10);
          emit stdout toString(1 <= i64val);
          wait(10);

          emit stdout toString('hello' <= 'world');
          wait(10);
          emit stdout toString('hello' <= 'hello');
          wait(10);

          emit exit i8val;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    LESSTHANEQUAL="true
false
true
false
true
false
true
false
true
false
true
false
true
true"

    It "runs js"
      When run test_js
      The output should eq "$LESSTHANEQUAL"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$LESSTHANEQUAL"
    End
  End

  Describe "Greater Than"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          const i8val = 0;
          emit stdout toString(i8val > 1);
          wait(10);
          emit stdout toString(1 > i8val);
          wait(10);

          const i16val: int16 = 0;
          emit stdout toString(i16val > 1);
          wait(10);
          emit stdout toString(1 > i16val);
          wait(10);

          const i32val: int32 = 0;
          emit stdout toString(i32val > 1);
          wait(10);
          emit stdout toString(1 > i32val);
          wait(10);

          const i64val: int64 = 0;
          emit stdout toString(i64val > 1);
          wait(10);
          emit stdout toString(1 > i64val);
          wait(10);

          emit stdout toString('hello' > 'world');
          wait(10);
          emit stdout toString('hello' > 'hello');
          wait(10);

          emit exit i8val;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GREATERTHAN="false
true
false
true
false
true
false
true
false
true
false
true
false
true"

    It "runs js"
      When run test_js
      The output should eq "$GREATERTHAN"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$GREATERTHAN"
    End
  End

  Describe "Greater Than Or Equal"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          const i8val = 0;
          emit stdout toString(i8val >= 1);
          wait(10);
          emit stdout toString(1 >= i8val);
          wait(10);

          const i16val: int16 = 0;
          emit stdout toString(i16val >= 1);
          wait(10);
          emit stdout toString(1 >= i16val);
          wait(10);

          const i32val: int32 = 0;
          emit stdout toString(i32val >= 1);
          wait(10);
          emit stdout toString(1 >= i32val);
          wait(10);

          const i64val: int64 = 0;
          emit stdout toString(i64val >= 1);
          wait(10);
          emit stdout toString(1 >= i64val);
          wait(10);

          emit stdout toString('hello' >= 'world');
          wait(10);
          emit stdout toString('hello' >= 'hello');
          wait(10);

          emit exit i8val;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GREATERTHANOREQUAL="true
false
true
false
true
false
true
false
true
false
true
false
true
false"

    It "runs js"
      When run test_js
      The output should eq "$GREATERTHANOREQUAL"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$GREATERTHANOREQUAL"
    End
  End
End
