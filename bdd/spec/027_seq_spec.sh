Include build_tools.sh

Describe "@std/seq"
  Describe "seq and next"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, next

        on start {
          let s = seq(2);
          print(s.next());
          print(s.next());
          print(s.next());
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    NEXTOUTPUT="0
1
error: sequence out-of-bounds"

    It "runs js"
      When run test_js
      The output should eq "$NEXTOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$NEXTOUTPUT"
    End
  End

  Describe "each"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, each

        on start {
          let s = seq(3);
          s.each(fn (i: int64) = print(i));
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EACHOUTPUT="0
1
2"

    It "runs js"
      When run test_js
      The output should eq "$EACHOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$EACHOUTPUT"
    End
  End

  Describe "while"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, while

        on start {
          let s = seq(100);
          let sum = 0;
          s.while(fn = sum < 10, fn {
            sum = sum + 1 || 0;
          });
          print(sum);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    WHILEOUTPUT="10"

    It "runs js"
      When run test_js
      The output should eq "$WHILEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$WHILEOUTPUT"
    End
  End

  Describe "do-while"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, doWhile

        on start {
          let s = seq(100);
          let sum = 0;
          // TODO: Get automatic type inference working on anonymous multi-line functions
          s.doWhile(fn (): bool {
            sum = sum + 1 || 0;
            return sum < 10;
          });
          print(sum);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    DOWHILEOUTPUT="10"

    It "runs js"
      When run test_js
      The output should eq "$DOWHILEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$DOWHILEOUTPUT"
    End
  End

  Describe "recurse"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, Self, recurse

        on start {
          print(seq(100).recurse(fn fibonacci(self: Self, i: int64): Result<int64> {
            if i < 2 {
              return ok(1);
            } else {
              const prev = self.recurse(i - 1 || 0);
              const prevPrev = self.recurse(i - 2 || 0);
              if prev.isErr() {
                return prev;
              }
              if prevPrev.isErr() {
                return prevPrev;
              }
              // TODO: Get type inference inside of recurse working so we don't need to unwrap these
              return (prev || 0) + (prevPrev || 0);
            }
          }, 8));
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    RECURSEOUTPUT="34"

    It "runs js"
      When run test_js
      The output should eq "$RECURSEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$RECURSEOUTPUT"
    End
  End

  Describe "recurse no-op one-liner regression test"
    # Reported issue -- the root cause was due to how the compiler handled one-liner functions
    # differently from multi-line functions. This test is to make sure the fix for this doesn't
    # regress
    before() {
      sourceToAll "
        import @std/app
        from @std/seq import seq, Self, recurse

        fn doNothing(x: int) : int = x;

        fn doNothingRec(x: int) : int = seq(x).recurse(fn (self: Self, x: int) : Result<int> {
            return ok(x);
        }, x) || 0;

        on app.start {
            const x = 5;
            app.print(doNothing(x)); // 5
            app.print(doNothingRec(x)); // 5

            const xs = [1, 2, 3];
            app.print(xs.map(doNothing).map(toString).join(' ')); // 1 2 3
            app.print(xs.map(doNothingRec).map(toString).join(' ')); // 1 2 3

            emit app.exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    ONELINEROUTPUT="5
5
1 2 3
1 2 3"

    It "runs js"
      When run test_js
      The output should eq "$ONELINEROUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$ONELINEROUTPUT"
    End
  End

  Describe "recurse decrement regression test and variable aliasing regression test"
    # Reported issue -- the root cause was two simultaneous bugs in the AVM and the ammtoaga stage
    # of the compiler. The AVM was not decrementing the seq counter correctly when inside of a
    # parallel opcode environment because it was being reset accidentally when merging the memory
    # changes. It was also accidentally obliterating one of its arguments but this was masked by a
    # bug in variable scope aliasing logic in the ammtoaga layer of the compiler (hence why the bug
    # was not seen in the JS path). This test case guards against both issues.
    before() {
      sourceToAll "
        import @std/app
        from @std/seq import seq, Self, recurse

        fn triangularRec(x: int) : int = seq(x + 1 || 0).recurse(fn (self: Self, x: int) : Result<int> {
          if x == 0 {
            return ok(x);
          } else {
            // TODO: Get type inference inside of recurse working so we don't need to unwrap these
            return x + (self.recurse(x - 1 || 0) || 0);
          }
        }, x) || 0

        on app.start {
          const xs = [1, 2, 3];
          app.print(xs.map(triangularRec).map(toString).join(' ')); // 1 3 6

          emit app.exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    DECREMENTOUTPUT="1 3 6"

    It "runs js"
      When run test_js
      The output should eq "$DECREMENTOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$DECREMENTOUTPUT"
    End
  End
End