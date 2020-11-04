Include build_tools.sh

Describe "Tree"
  Describe "basic construction and access"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const t = newTree('foo').addChild(
            newTree('bar').addChild('bay')
          ).addChild('baz').getTree()

          print(t.getRootNode() || 'wrong')
          print(t.getChildren().map(fn (c: Node<string>): string = c || 'wrong').join(', '))

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    BASICOUTPUT="foo
bar, baz"

    It "runs js"
      When run node temp.js
      The output should eq "$BASICOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$BASICOUTPUT"
    End
  End
End
