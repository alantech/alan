Include build_tools.sh

Describe "Comparators"
  Describe "Equals"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          const i8val = 0;
          emit stdout concat(toString(eq(i8val, 0)), '\n');
          emit stdout concat(toString(eq(i8val, 1)), '\n');

          const i16val: int16 = 0;
          emit stdout concat(toString(eq(i16val, 0)), '\n');
          emit stdout concat(toString(eq(i16val, 1)), '\n');

          const i32val: int32 = 0;
          emit stdout concat(toString(eq(i32val, 0)), '\n');
          emit stdout concat(toString(eq(i32val, 1)), '\n');

          const i64val: int64 = 0;
          emit stdout concat(toString(eq(i64val, 0)), '\n');
          emit stdout concat(toString(eq(i64val, 1)), '\n');

          const f32val: float32 = 0.0;
          emit stdout concat(toString(eq(f32val, 0.0)), '\n');
          emit stdout concat(toString(eq(f32val, 0.1)), '\n');

          const f64val: float64 = 0.0;
          emit stdout concat(toString(eq(f64val, 0.0)), '\n');
          emit stdout concat(toString(eq(f64val, 0.1)), '\n');

          emit stdout concat(toString(eq(true, true)), '\n');
          emit stdout concat(toString(eq(true, false)), '\n');

          emit stdout concat(toString(eq('hello', \"hello\")), '\n');
          emit stdout concat(toString(eq('hello', \"world\")), '\n');

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
  End
End
