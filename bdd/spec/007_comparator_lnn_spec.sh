Include build_tools.sh

Describe "Comparators"
  Describe "Equals"
    before() {
      lnn_sourceToTemp "
        from @std/app import start, stdout, exit

        on start {
          // constrained to an int8 from the emit exit at the bottom
          const i8val = 0;
          const i8val1: int8 = 0;
          emit stdout toString(i8val == i8val1) + '\n';
          wait(10);
          const i8val2: int8 = 1;
          emit stdout toString(i8val == i8val2) + '\n';
          wait(10);

          const i16val: int16 = 0;
          const i16val1: int16 = 0;
          emit stdout toString(i16val == i16val1) + '\n';
          wait(10);
          const i16val2: int16 = 1;
          emit stdout toString(i16val == i16val2) + '\n';
          wait(10);

          const i32val: int32 = 0;
          const i32val1: int32 = 0;
          emit stdout toString(i32val == i32val1) + '\n';
          wait(10);
          const i32val2: int32 = 1;
          emit stdout toString(i32val == i32val2) + '\n';
          wait(10);

          const i64val: int64 = 0;
          const i64val1: int64 = 0;
          emit stdout toString(i64val == i64val1) + '\n';
          wait(10);
          const i64val2: int64 = 1;
          emit stdout toString(i64val == i64val2) + '\n';
          wait(10);

          const f32val: float32 = 0.0;
          const f32val1: float32 = 0.0;
          emit stdout toString(f32val == f32val1) + '\n';
          wait(10);
          const f32val2: float32 = 1.0;
          emit stdout toString(f32val == f32val2) + '\n';
          wait(10);

          const f64val: float64 = 0.0;
          const f64val1: float64 = 0.0;
          emit stdout toString(f64val == f64val1) + '\n';
          wait(10);
          const f64val2: float64 = 1.0;
          emit stdout toString(f64val == f64val2) + '\n';
          wait(10);

          emit stdout toString(true == true) + '\n';
          wait(10);
          emit stdout toString(true == false) + '\n';
          wait(10);

          emit stdout toString('hello' == \"hello\") + '\n';
          wait(10);
          emit stdout toString('hello' == \"world\") + '\n';
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
      When run test_js
      The output should eq "$EQUALS"
    End

    It "runs agc"
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
          const i8val1: int8 = 0;
          emit stdout toString(i8val != i8val1) + '\n';
          wait(10);
          const i8val2: int8 = 1;
          emit stdout toString(i8val != i8val2) + '\n';
          wait(10);

          const i16val: int16 = 0;
          const i16val1: int16 = 0;
          emit stdout toString(i16val != i16val1) + '\n';
          wait(10);
          const i16val2: int16 = 1;
          emit stdout toString(i16val != i16val2) + '\n';
          wait(10);

          const i32val: int32 = 0;
          const i32val1: int32 = 0;
          emit stdout toString(i32val != i32val1) + '\n';
          wait(10);
          const i32val2: int32 = 1;
          emit stdout toString(i32val != i32val2) + '\n';
          wait(10);

          const i64val: int64 = 0;
          const i64val1: int64 = 0;
          emit stdout toString(i64val != i64val1) + '\n';
          wait(10);
          const i64val2: int64 = 1;
          emit stdout toString(i64val != i64val2) + '\n';
          wait(10);

          const f32val: float32 = 0;
          const f32val1: float32 = 0.0;
          emit stdout toString(f32val != f32val1) + '\n';
          wait(10);
          const f32val2: float32 = 1.0;
          emit stdout toString(f32val != f32val2) + '\n';
          wait(10);

          const f64val: float64 = 0;
          const f64val1: float64 = 0.0;
          emit stdout toString(f64val != f64val1) + '\n';
          wait(10);
          const f64val2: float64 = 1.0;
          emit stdout toString(f64val != f64val2) + '\n';
          wait(10);

          emit stdout toString(true != true) + '\n';
          wait(10);
          emit stdout toString(true != false) + '\n';
          wait(10);

          emit stdout toString(\"hello\" != \"hello\") + '\n';
          wait(10);
          emit stdout toString(\"hello\" != \"world\") + '\n';
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
          const otheri8: int8 = 1;
          emit stdout toString(i8val < otheri8) + '\n';
          wait(10);
          emit stdout toString(otheri8 < i8val) + '\n';
          wait(10);

          const i16val: int16 = 0;
          const otheri16: int16 = 1;
          emit stdout toString(i16val < otheri16) + '\n';
          wait(10);
          emit stdout toString(otheri16 < i16val) + '\n';
          wait(10);

          const i32val: int32 = 0;
          const otheri32: int32 = 1;
          emit stdout toString(i32val < otheri32) + '\n';
          wait(10);
          emit stdout toString(otheri32 < i32val) + '\n';
          wait(10);

          const i64val: int64 = 0;
          const otheri64: int64 = 1;
          emit stdout toString(i64val < otheri64) + '\n';
          wait(10);
          emit stdout toString(otheri64 < i64val) + '\n';
          wait(10);

          emit stdout toString('hello' < 'world') + '\n';
          wait(10);
          emit stdout toString('hello' < 'hello') + '\n';
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
          const otheri8: int8 = 1;
          emit stdout toString(i8val <= otheri8) + '\n';
          wait(10);
          emit stdout toString(otheri8 <= i8val) + '\n';
          wait(10);

          const i16val: int16 = 0;
          const otheri16: int16 = 1;
          emit stdout toString(i16val <= otheri16) + '\n';
          wait(10);
          emit stdout toString(otheri16 <= i16val) + '\n';
          wait(10);

          const i32val: int32 = 0;
          const otheri32: int32 = 1;
          emit stdout toString(i32val <= otheri32) + '\n';
          wait(10);
          emit stdout toString(otheri32 <= i32val) + '\n';
          wait(10);

          const i64val: int64 = 0;
          const otheri64: int64 = 1;
          emit stdout toString(i64val <= otheri64) + '\n';
          wait(10);
          emit stdout toString(otheri64 <= i64val) + '\n';
          wait(10);

          emit stdout toString('hello' <= 'world') + '\n';
          wait(10);
          emit stdout toString('hello' <= 'hello') + '\n';
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
          const otheri8: int8 = 1;
          emit stdout toString(i8val > otheri8) + '\n';
          wait(10);
          emit stdout toString(otheri8 > i8val) + '\n';
          wait(10);

          const i16val: int16 = 0;
          const otheri16: int16 = 1;
          emit stdout toString(i16val > otheri16) + '\n';
          wait(10);
          emit stdout toString(otheri16 > i16val) + '\n';
          wait(10);

          const i32val: int32 = 0;
          const otheri32: int32 = 1;
          emit stdout toString(i32val > otheri32) + '\n';
          wait(10);
          emit stdout toString(otheri32 > i32val) + '\n';
          wait(10);

          const i64val: int64 = 0;
          const otheri64: int64 = 1;
          emit stdout toString(i64val > otheri64) + '\n';
          wait(10);
          emit stdout toString(otheri64 > i64val) + '\n';
          wait(10);

          emit stdout toString('world' > 'world') + '\n';
          wait(10);
          emit stdout toString('world' > 'hello') + '\n';
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
          const otheri8: int8 = 1;
          emit stdout toString(i8val >= otheri8) + '\n';
          wait(10);
          emit stdout toString(otheri8 >= i8val) + '\n';
          wait(10);

          const i16val: int16 = 0;
          const otheri16: int16 = 1;
          emit stdout toString(i16val >= otheri16) + '\n';
          wait(10);
          emit stdout toString(otheri16 >= i16val) + '\n';
          wait(10);

          const i32val: int32 = 0;
          const otheri32: int32 = 1;
          emit stdout toString(i32val >= otheri32) + '\n';
          wait(10);
          emit stdout toString(otheri32 >= i32val) + '\n';
          wait(10);

          const i64val: int64 = 0;
          const otheri64: int64 = 1;
          emit stdout toString(i64val >= otheri64) + '\n';
          wait(10);
          emit stdout toString(otheri64 >= i64val) + '\n';
          wait(10);

          emit stdout toString('hello' >= 'world') + '\n';
          wait(10);
          emit stdout toString('hello' >= 'hello') + '\n';
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

    GREATERTHANOREQUAL="false
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
      The output should eq "$GREATERTHANOREQUAL"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$GREATERTHANOREQUAL"
    End
  End
End
