Include build_tools.sh

Describe "Bitwise"
  Describe "int8"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        prefix ~ 10 toInt8

        on start {
          print(~1 & ~2)
          print(~1 | ~3)
          print(~5 ^ ~3)
          print(! ~0)
          print(~1 !& ~2)
          print(~1 !| ~2)
          print(~5 !^ ~3)
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
      When run alan interpret temp.ln
      The output should eq "0
3
6
-1
-1
-4
-7"
    End
  End

  Describe "int16"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        prefix ~ 10 toInt16

        on start {
          print(~1 & ~2)
          print(~1 | ~3)
          print(~5 ^ ~3)
          print(! ~0)
          print(~1 !& ~2)
          print(~1 !| ~2)
          print(~5 !^ ~3)
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
      When run alan interpret temp.ln
      The output should eq "0
3
6
-1
-1
-4
-7"
    End
  End

  Describe "int32"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        prefix ~ 10 toInt32

        on start {
          print(~1 & ~2)
          print(~1 | ~3)
          print(~5 ^ ~3)
          print(! ~0)
          print(~1 !& ~2)
          print(~1 !| ~2)
          print(~5 !^ ~3)
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
      When run alan interpret temp.ln
      The output should eq "0
3
6
-1
-1
-4
-7"
    End
  End

  Describe "int64"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(1 & 2)
          print(1 | 3)
          print(5 ^ 3)
          print(!0)
          print(1 !& 2)
          print(1 !| 2)
          print(5 !^ 3)
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
      When run alan interpret temp.ln
      The output should eq "0
3
6
-1
-1
-4
-7"
    End
  End
End
