Include build_tools.sh

Describe "Tree"
  Describe "basic construction and access"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const myTree = newTree('foo')
          const barNode = myTree.addChild('bar')
          const bazNode = myTree.addChild('baz')
          const bayNode = barNode.addChild('bay')

          print(myTree.getRootNode() || 'wrong')
          print(myTree.getChildren().map(fn (c: Node<string>): string = c || 'wrong').join(', '))

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

  Describe "every, find, some, reduce and prune"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const myTree = newTree('foo')
          const barNode = myTree.addChild('bar')
          const bazNode = myTree.addChild('baz')
          const bayNode = barNode.addChild('bay')

          print(myTree.every(fn (c: Node<string>): bool = (c || 'wrong').length() == 3))
          print(myTree.some(fn (c: Node<string>): bool = (c || 'wrong').length() == 1))
          print(myTree.find(fn (c: Node<string>): bool = (c || 'wrong') == 'bay').getOr('wrong'))
          print(myTree.find(fn (c: Node<string>): bool = (c || 'wrong') == 'asf').getOr('wrong'))

          print(myTree.length())
          myTree.getChildren().eachLin(fn (c: Node<string>) {
            const n = c || 'wrong'
            if n == 'bar' {
              c.prune()
            }
          })
          print(myTree.getChildren().map(fn (c: Node<string>): string = c || 'wrong').join(', '))
          print(myTree.length())

          myTree.reduce(fn (acc: int, i: Node<string>): int = (i || 'wrong').length() + acc, 0).print()
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    BASICOUTPUT="true
false
bay
wrong
4
2
6"

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
