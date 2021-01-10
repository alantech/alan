Include build_tools.sh

Describe "Module imports"
  Describe "can import with trailing whitespace"
    before() {
      sourceToFile piece.ln "
        export type Piece {
          owner: bool,
        }
      "
      sourceToAll "
        from @std/app import start, print, exit
        // Intentionally put an extra space after the import
        from ./piece import Piece 

        on start {
          const piece = new Piece {
            owner: false,
          };
          print('Hello, World!');
          if piece.owner == true {
            print('OK');
          } else {
            print('False');
          }
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "Hello, World!
False"
    End

    It "runs agc"
      When run test_agc
      The output should eq "Hello, World!
False"
    End
  End
End