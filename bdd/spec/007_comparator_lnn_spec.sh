Include build_tools.sh

Describe "Comparators"
  Describe "Equals"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          // constrained to an int8 from the emit exit at the bottom
          const i8val = 0;
          emit stdout concat(toString(eq(i8val, 0)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(i8val, 1)), '\n');
          wait(1000);

          const i16val: int16 = 0;
          emit stdout concat(toString(eq(i16val, 0)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(i16val, 1)), '\n');
          wait(1000);

          const i32val: int32 = 0;
          emit stdout concat(toString(eq(i32val, 0)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(i32val, 1)), '\n');
          wait(1000);

          const i64val: int64 = 0;
          emit stdout concat(toString(eq(i64val, 0)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(i64val, 1)), '\n');
          wait(1000);

          const f32val: float32 = 0.0;
          emit stdout concat(toString(eq(f32val, 0.0)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(f32val, 0.1)), '\n');
          wait(1000);

          const f64val: float64 = 0.0;
          emit stdout concat(toString(eq(f64val, 0.0)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(f64val, 0.1)), '\n');
          wait(1000);

          emit stdout concat(toString(eq(true, true)), '\n');
          wait(1000);
          emit stdout concat(toString(eq(true, false)), '\n');
          wait(1000);

          emit stdout concat(toString(eq('hello', \"hello\")), '\n');
          wait(1000);
          emit stdout concat(toString(eq('hello', \"world\")), '\n');

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
      When run test_js
      The output should eq "$EQUALS"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$EQUALS"
    End
  End
End
