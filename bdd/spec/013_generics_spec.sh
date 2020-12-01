Include build_tools.sh

Describe "Generics"
  Describe "valid generic type definition"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        type box<V> {
          set: bool
          val: V
        }

        on start fn {
          let int8Box = new box<int8> {
            val = 8.toInt8()
            set = true
          }
          print(int8Box.val)
          print(int8Box.set)

          let stringBox = new box<string> {
            val = 'hello, generics!'
            set = true
          }
          print(stringBox.val)
          print(stringBox.set)

          const stringBoxBox = new box<box<string>> {
            val = new box<string> {
              val = 'hello, nested generics!'
              set = true
            }
            set = true
          }
          // TODO: This was originally 'stringBoxBox.set.print()' but that syntax doesn't work yet
          print(stringBoxBox.set)
          print(stringBoxBox.val.set)
          print(stringBoxBox.val.val)

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GENERICOUTPUT="8
true
hello, generics!
true
true
true
hello, nested generics!"

    It "runs js"
      When run test_js
      The output should eq "$GENERICOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$GENERICOUTPUT"
    End
  End

  Describe "invalid generic usage"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        type box<V> {
          set: bool
          val: V
        }

        on start fn {
          let stringBox = new box<string> {
            set = true
            val = 'str'
          }
          stringBox.val = 8

          emit exit 0
        }
      "
    }
    Before before

    after() {
      cleanTemp
    }
    After after

    It "does not compile"
      When run alan compile test_$$/temp.ln test_$$/temp.agc
      The error should eq "stringBox.val is of type string but assigned a value of type int64"
      The status should not eq "0"
    End
  End
End
