Include build_tools.sh

Describe "Comparators"
  Describe "Cross-type comparisons"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(true == 1)
          emit exit 0
        }
      "
    }
    Before before

    after() {
      cleanTemp
    }
    After after

    It "doesn't work"
      When run alan-interpreter interpret temp.ln
      The status should not eq "0"
      The error should eq "Unable to find matching function for name and argument type set"
    End
  End

  Describe "Equals"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) == toInt8(0))
          print(toInt8(1).eq(toInt8(0)))

          print(toInt16(0) == toInt16(0))
          print(toInt16(1).eq(toInt16(0)))

          print(toInt32(0) == toInt32(0))
          print(toInt32(1).eq(toInt32(0)))

          print(0 == 0)
          print(1.eq(0))

          print(toFloat32(0.0) == toFloat32(0.0))
          print(toFloat32(1.2).eq(toFloat32(0.0)))

          print(0.0 == 0.0)
          print(1.2.eq(0.0))

          print(true == true)
          print(true.eq(false))

          print('hello' == 'hello')
          print('hello'.eq('world'))

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
      The output should eq "true
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

  Describe "Not Equals"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) != toInt8(0))
          print(toInt8(1).neq(toInt8(0)))

          print(toInt16(0) != toInt16(0))
          print(toInt16(1).neq(toInt16(0)))

          print(toInt32(0) != toInt32(0))
          print(toInt32(1).neq(toInt32(0)))

          print(0 != 0)
          print(1.neq(0))

          print(toFloat32(0.0) != toFloat32(0.0))
          print(toFloat32(1.2).neq(toFloat32(0.0)))

          print(0.0 != 0.0)
          print(1.2.neq(0.0))

          print(true != true)
          print(true.neq(false))

          print('hello' != 'hello')
          print('hello'.neq('world'))

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
      The output should eq "false
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
    End
  End

  Describe "Less Than"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) < toInt8(1))
          print(toInt8(1).lt(toInt8(0)))

          print(toInt16(0) < toInt16(1))
          print(toInt16(1).lt(toInt16(0)))

          print(toInt32(0) < toInt32(1))
          print(toInt32(1).lt(toInt32(0)))

          print(0 < 1)
          print(1.lt(0))

          print(toFloat32(0.0) < toFloat32(1.0))
          print(toFloat32(1.2).lt(toFloat32(0.0)))

          print(0.0 < 1.0)
          print(1.2.lt(0.0))

          print('hello' < 'hello')
          print('hello'.lt('world'))

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
      The output should eq "true
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
false
true"
    End
  End

  Describe "Less Than Or Equal"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) <= toInt8(1))
          print(toInt8(1).lte(toInt8(0)))

          print(toInt16(0) <= toInt16(1))
          print(toInt16(1).lte(toInt16(0)))

          print(toInt32(0) <= toInt32(1))
          print(toInt32(1).lte(toInt32(0)))

          print(0 <= 1)
          print(1.lte(0))

          print(toFloat32(0.0) <= toFloat32(1.0))
          print(toFloat32(1.2).lte(toFloat32(0.0)))

          print(0.0 <= 1.0)
          print(1.2.lte(0.0))

          print('hello' <= 'hello')
          print('hello'.lte('world'))

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
      The output should eq "true
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
    End
  End

  Describe "Greater Than"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) > toInt8(1))
          print(toInt8(1).gt(toInt8(0)))

          print(toInt16(0) > toInt16(1))
          print(toInt16(1).gt(toInt16(0)))

          print(toInt32(0) > toInt32(1))
          print(toInt32(1).gt(toInt32(0)))

          print(0 > 1)
          print(1.gt(0))

          print(toFloat32(0.0) > toFloat32(1.0))
          print(toFloat32(1.2).gt(toFloat32(0.0)))

          print(0.0 > 1.0)
          print(1.2.gt(0.0))

          print('hello' > 'hello')
          print('hello'.gt('world'))

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
      The output should eq "false
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
false"
    End
  End

  Describe "Greater Than Or Equal"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) >= toInt8(1))
          print(toInt8(1).gte(toInt8(0)))

          print(toInt16(0) >= toInt16(1))
          print(toInt16(1).gte(toInt16(0)))

          print(toInt32(0) >= toInt32(1))
          print(toInt32(1).gte(toInt32(0)))

          print(0 >= 1)
          print(1.gte(0))

          print(toFloat32(0.0) >= toFloat32(1.0))
          print(toFloat32(1.2).gte(toFloat32(0.0)))

          print(0.0 >= 1.0)
          print(1.2.gte(0.0))

          print('hello' >= 'hello')
          print('hello'.gte('world'))

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
      The output should eq "false
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
true
false"
    End
  End
End

