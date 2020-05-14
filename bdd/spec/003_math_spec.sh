Include build_tools.sh

Describe "Basic Math"
  Describe "int8 (not default)"
    Describe "addition"
      before() {
        sourceToTemp "
          from @std/app import start, exit
          on start { emit exit add(toInt8(1), toInt8(2)) }
        "
      }
      Before before

      after() {
        cleanTemp
      }
      After after

      It "interprets"
        When run alan-interpreter interpret temp.ln
        The status should eq "3"
      End
    End
    
    Describe "subtraction"
      before() {
        sourceToTemp "
          from @std/app import start, exit
          on start { emit exit sub(toInt8(2), toInt8(1)) }
        "
      }
      Before before

      after() {
        cleanTemp
      }
      After after

      It "interprets"
        When run alan-interpreter interpret temp.ln
        The status should eq "1"
      End
    End
    
    Describe "multiplication"
      before() {
        sourceToTemp "
          from @std/app import start, exit
          on start { emit exit mul(toInt8(2), toInt8(1)) }
        "
      }
      Before before

      after() {
        cleanTemp
      }
      After after

      It "interprets"
        When run alan-interpreter interpret temp.ln
        The status should eq "2"
      End
    End
    
    Describe "division"
      before() {
        sourceToTemp "
          from @std/app import start, exit
          on start { emit exit div(toInt8(6), toInt8(2)) }
        "
      }
      Before before

      after() {
        cleanTemp
      }
      After after

      It "interprets"
        When run alan-interpreter interpret temp.ln
        The status should eq "3"
      End
    End
    
    Describe "modulus"
      before() {
        sourceToTemp "
          from @std/app import start, exit
          on start { emit exit mod(toInt8(6), toInt8(4)) }
        "
      }
      Before before

      after() {
        cleanTemp
      }
      After after

      It "interprets"
        When run alan-interpreter interpret temp.ln
        The status should eq "2"
      End
    End
    
    Describe "exponentiation"
      before() {
        sourceToTemp "
          from @std/app import start, exit
          on start { emit exit pow(toInt8(6), toInt8(2)) }
        "
      }
      Before before

      after() {
        cleanTemp
      }
      After after

      It "interprets"
        When run alan-interpreter interpret temp.ln
        The status should eq "36"
      End
    End
  End

  Describe "int16 (not default)"
    Describe "addition"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(add(toInt16(1), toInt16(2)))
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
        The output should eq "3"
      End
    End
    
    Describe "subtraction"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(sub(toInt16(2), toInt16(1)))
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
        The output should eq "1"
      End
    End
    
    Describe "multiplication"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(mul(toInt16(2), toInt16(1)))
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
        The output should eq "2"
      End
    End
    
    Describe "division"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(div(toInt16(6), toInt16(2)))
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
        The output should eq "3"
      End
    End
    
    Describe "modulus"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(mod(toInt16(6), toInt16(4)))
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
        The output should eq "2"
      End
    End
    
    Describe "exponentiation"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(pow(toInt16(6), toInt16(2)))
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
        The output should eq "36"
      End
    End
  End

  Describe "int32 (not default)"
    Describe "addition"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            add(1.toInt32(), 2.toInt32()).print()
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
        The output should eq "3"
      End
    End
    
    Describe "subtraction"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            sub(2.toInt32(), 1.toInt32()).print()
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
        The output should eq "1"
      End
    End
    
    Describe "multiplication"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            mul(2.toInt32(), 1.toInt32()).print()
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
        The output should eq "2"
      End
    End
    
    Describe "division"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            div(6.toInt32(), 2.toInt32()).print()
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
        The output should eq "3"
      End
    End
    
    Describe "modulus"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            mod(6.toInt32(), 4.toInt32()).print()
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
        The output should eq "2"
      End
    End
    
    Describe "exponentiation"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            pow(6.toInt32(), 2.toInt32()).print()
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
        The output should eq "36"
      End
    End
  End

  Describe "int64 (default)"
    Describe "addition"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(1 + 2)
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
        The output should eq "3"
      End
    End
    
    Describe "subtraction"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(2 - 1)
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
        The output should eq "1"
      End
    End
    
    Describe "multiplication"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(2 * 1)
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
        The output should eq "2"
      End
    End
    
    Describe "division"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(6 / 2)
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
        The output should eq "3"
      End
    End
    
    Describe "modulus"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(6 % 4)
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
        The output should eq "2"
      End
    End
    
    Describe "exponentiation"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(6 ** 2)
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
        The output should eq "36"
      End
    End
  End

  Describe "float32 (not default)"
    Describe "addition"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(1) + toFloat32(2))
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
        The output should eq "3.0"
      End
    End
    
    Describe "subtraction"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(2) - toFloat32(1))
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
        The output should eq "1.0"
      End
    End
    
    Describe "multiplication"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(2) * toFloat32(1))
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
        The output should eq "2.0"
      End
    End
    
    Describe "division"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(6) / toFloat32(2))
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
        The output should eq "3.0"
      End
    End
    
    Describe "sqrt"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(sqrt(toFloat32(36)))
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
        The output should eq "6.0"
      End
    End
    
    Describe "exponentiation"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(6) ** toFloat32(2))
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
        The output should eq "36.0"
      End
    End
  End

  Describe "float64 (default)"
    Describe "addition"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            (1.0 + 2.0).print()
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
        The output should eq "3.0"
      End
    End
    
    Describe "subtraction"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            (2.0 - 1.0).print()
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
        The output should eq "1.0"
      End
    End
    
    Describe "multiplication"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            (2.0 * 1.0).print()
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
        The output should eq "2.0"
      End
    End
    
    Describe "division"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            (6.0 / 2.0).print()
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
        The output should eq "3.0"
      End
    End
    
    Describe "sqrt"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            sqrt(36.0).print()
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
        The output should eq "6.0"
      End
    End
    
    Describe "exponentiation"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            (6.0 ** 2.0).print()
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
        The output should eq "36.0"
      End
    End
  End
End
