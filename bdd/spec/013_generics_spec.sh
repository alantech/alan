Include build_tools.sh

Describe "Generics"
  Describe "valid generic type definition"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        type box<V> {
          set: bool
          val: V
        }

        on start fn {
          let int8Box: box<int8>
          int8Box.val = 8
          int8Box.set = true
          print(int8Box.val)
          print(int8Box.set)

          let stringBox: box<string>
          stringBox.val = 'hello, generics!'
          stringBox.set = true
          print(stringBox.val)
          print(stringBox.set)

          const stringBoxBox = new box<box<string>> {
            val = new box<string> {
              val = 'hello, nested generics!'
              set = true
            }
            set = true
          }
          stringBoxBox.set.print()
          stringBoxBox.val.set.print()
          stringBoxBox.val.val.print()

          emit exit 0
        }
      "
    }
    Before before

    after() {
      cleanTemp
    }
    After after

    It "interprets"
      When run alan-interpreter interpret temp.ln
      The output should eq "8
true
hello, generics!
true
true
true
hello, nested generics!"
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
          let stringBox: box<string>
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

    It "does not interpret"
      When run alan-interpreter interpret temp.ln
      The error should eq "Assigning integer number to non-numeric type
Variable type: string"
      The status should not eq "0"
    End
  End      
End
