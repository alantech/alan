Include build_tools.sh

Describe "Comparators"
  Describe "Cross-type comparisons"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(true == 1);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
      The status should not eq "0"
      The error should eq "Cannot resolve operators with remaining statement
true == 1
<bool> == <int64>"
    End
  End

  Describe "Equals"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) == toInt8(0));
          print(toInt8(1).eq(toInt8(0)));

          print(toInt16(0) == toInt16(0));
          print(toInt16(1).eq(toInt16(0)));

          print(toInt32(0) == toInt32(0));
          print(toInt32(1).eq(toInt32(0)));

          print(0 == 0);
          print(1.eq(0));

          print(toFloat32(0.0) == toFloat32(0.0));
          print(toFloat32(1.2).eq(toFloat32(0.0)));

          print(0.0 == 0.0);
          print(1.2.eq(0.0));

          print(true == true);
          print(true.eq(false));

          print('hello' == 'hello');
          print('hello'.eq('world'));

          emit exit 0;
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
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) != toInt8(0));
          print(toInt8(1).neq(toInt8(0)));

          print(toInt16(0) != toInt16(0));
          print(toInt16(1).neq(toInt16(0)));

          print(toInt32(0) != toInt32(0));
          print(toInt32(1).neq(toInt32(0)));

          print(0 != 0);
          print(1.neq(0));

          print(toFloat32(0.0) != toFloat32(0.0));
          print(toFloat32(1.2).neq(toFloat32(0.0)));

          print(0.0 != 0.0);
          print(1.2.neq(0.0));

          print(true != true);
          print(true.neq(false));

          print('hello' != 'hello');
          print('hello'.neq('world'));

          emit exit 0;
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
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) < toInt8(1));
          print(toInt8(1).lt(toInt8(0)));

          print(toInt16(0) < toInt16(1));
          print(toInt16(1).lt(toInt16(0)));

          print(toInt32(0) < toInt32(1));
          print(toInt32(1).lt(toInt32(0)));

          print(0 < 1);
          print(1.lt(0));

          print(toFloat32(0.0) < toFloat32(1.0));
          print(toFloat32(1.2).lt(toFloat32(0.0)));

          print(0.0 < 1.0);
          print(1.2.lt(0.0));

          print('hello' < 'hello');
          print('hello'.lt('world'));

          emit exit 0;
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
false
true
false
false
true"

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
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) <= toInt8(1));
          print(toInt8(1).lte(toInt8(0)));

          print(toInt16(0) <= toInt16(1));
          print(toInt16(1).lte(toInt16(0)));

          print(toInt32(0) <= toInt32(1));
          print(toInt32(1).lte(toInt32(0)));

          print(0 <= 1);
          print(1.lte(0));

          print(toFloat32(0.0) <= toFloat32(1.0));
          print(toFloat32(1.2).lte(toFloat32(0.0)));

          print(0.0 <= 1.0);
          print(1.2.lte(0.0));

          print('hello' <= 'hello');
          print('hello'.lte('world'));

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    LESSTHANOREQUAL="true
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

    It "runs js"
      When run test_js
      The output should eq "$LESSTHANOREQUAL"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$LESSTHANOREQUAL"
    End
  End

  Describe "Greater Than"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) > toInt8(1));
          print(toInt8(1).gt(toInt8(0)));

          print(toInt16(0) > toInt16(1));
          print(toInt16(1).gt(toInt16(0)));

          print(toInt32(0) > toInt32(1));
          print(toInt32(1).gt(toInt32(0)));

          print(0 > 1);
          print(1.gt(0));

          print(toFloat32(0.0) > toFloat32(1.0));
          print(toFloat32(1.2).gt(toFloat32(0.0)));

          print(0.0 > 1.0);
          print(1.2.gt(0.0));

          print('hello' > 'hello');
          print('hello'.gt('world'));

          emit exit 0;
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
true
false
true
false
false"

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
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toInt8(0) >= toInt8(1));
          print(toInt8(1).gte(toInt8(0)));

          print(toInt16(0) >= toInt16(1));
          print(toInt16(1).gte(toInt16(0)));

          print(toInt32(0) >= toInt32(1));
          print(toInt32(1).gte(toInt32(0)));

          print(0 >= 1);
          print(1.gte(0));

          print(toFloat32(0.0) >= toFloat32(1.0));
          print(toFloat32(1.2).gte(toFloat32(0.0)));

          print(0.0 >= 1.0);
          print(1.2.gte(0.0));

          print('hello' >= 'hello');
          print('hello'.gte('world'));

          emit exit 0;
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
true
false
true
true
false"

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

