Include build_tools.sh

Describe "@std/seq"
  Describe "seq and next"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, next

        on start {
          let s = seq(2)
          print(s.next())
          print(s.next())
          print(s.next())
          emit exit 0
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
      When run node temp.js
      The output should eq "$NEXTOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$NEXTOUTPUT"
    End
  End

  Describe "each"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, each

        on start {
          let s = seq(3)
          s.each(fn (i: int64) = print(i))
          emit exit 0
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
      When run node temp.js
      The output should eq "$EACHOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$EACHOUTPUT"
    End
  End

  Describe "while"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, while

        on start {
          let s = seq(100)
          let sum = 0
          // TODO: Get type inference working for one-liner closures
          s.while(fn (): bool = sum < 10, fn {
            sum = sum + 1
          })
          print(sum)
          emit exit 0
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
      When run node temp.js
      The output should eq "$WHILEOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$WHILEOUTPUT"
    End
  End

  Describe "do-while"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/seq import seq, doWhile

        on start {
          let s = seq(100)
          let sum = 0
          // TODO: Get type inference working for one-liner closures
          s.doWhile(fn (): bool {
            sum = sum + 1
            return sum < 10
          })
          print(sum)
          emit exit 0
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
      When run node temp.js
      The output should eq "$DOWHILEOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
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
              return ok(1)
            } else {
              const prev = self.recurse(i - 1)
              const prevPrev = self.recurse(i - 2)
              if prev.isErr() {
                return prev
              }
              if prevPrev.isErr() {
                return prevPrev
              }
              return ok((prev || 1) + (prevPrev || 1))
            }
          }, 8))
          emit exit 0
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
      When run node temp.js
      The output should eq "$RECURSEOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
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

        fn doNothing(x: int) : int = x

        fn doNothingRec(x: int) : int = seq(x).recurse(fn (self: Self, x: int) : Result<int> {
            return ok(x)
        }, x) || 0

        on app.start {
            const x = 5
            app.print(doNothing(x)) // 5
            app.print(doNothingRec(x)) // 5

            const xs = [1, 2, 3]
            app.print(xs.map(doNothing).map(toString).join(' ')) // 1 2 3
            app.print(xs.map(doNothingRec).map(toString).join(' ')) // 1 2 3

            emit app.exit 0
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
      When run node temp.js
      The output should eq "$ONELINEROUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$ONELINEROUTPUT"
    End
  End
End