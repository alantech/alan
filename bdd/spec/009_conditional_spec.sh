Include build_tools.sh

Describe "Condiionals"
  Describe "Basic"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn bar() {
          print('bar!')
        }

        fn baz() {
          print('baz!')
        }

        on start {
          if 1 == 0 {
            print('What!?')
          } else {
            print('Math is sane...')
          }

          if 1 == 0 {
            print('Not this again...')
          } else if 1 == 2 {
            print('Still wrong...')
          } else {
            print('Math is still sane, for now...')
          }

          const foo: bool = true
          if foo bar else baz

          const isTrue = true
          cond(isTrue, fn {
            print(\"It's true!\")
          })
          cond(!isTrue, fn {
            print('This should not have run')
          })

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "Math is sane...
Math is still sane, for now...
bar!
It's true!"
    End

    It "runs agc"
      Pending condfn-regression-debugging
      When run alan-runtime run temp.agc
      The output should eq "Math is sane...
Math is still sane, for now...
bar!
It's true!"
    End
  End

  Describe "Advanced"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        fn nearOrFar(distance: float64): string {
          if distance < 5.0 {
            return 'Near!'
          } else {
            return 'Far!'
          }
        }

        on start {
          print(nearOrFar(3.14))
          print(nearOrFar(6.28))

          const options = pair(2, 4)
          print(options[0])
          print(options[1])

          const options2 = 3 : 5
          print(options2[0])
          print(options2[1])

          const val1 = 1 == 1 ? 1 : 2
          const val2 = 1 == 0 ? 1 : 2
          print(val1)
          print(val2)

          const val3 = cond(1 == 1, pair(3, 4))
          const val4 = cond(1 == 0, pair(3, 4))
          print(val3)
          print(val4)

          const val5 = 1 == 0 ? options2
          print(val5)

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
      The output should eq "Near!
Far!
2
4
3
5
1
2
3
4
5"
    End
  End
End