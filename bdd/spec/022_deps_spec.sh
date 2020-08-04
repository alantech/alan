Include build_tools.sh

Describe "@std/deps"
  before() {
    sourceToAll "
      from @std/deps import install, add, commit

      on install {
        add('git+ssh://git@github.com/alantech/hellodep')
        commit()
      }
    "
    # The following is to avoid unexpected output from SSH during the tests
    git clone git+ssh://git@github.com/alantech/hellodep 2>/dev/null 1>/dev/null
    rm -r hellodep
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  after_each() {
    rm -r ./dependencies
  }
  After after_each

  has_dependencies() {
    test -d "./dependencies"
  }

  has_alantech() {
    test -d "./dependencies/alantech"
  }

  has_hellodep() {
    test -d "./dependencies/alantech/hellodep"
  }

  has_index() {
    test -f "./dependencies/alantech/hellodep/index.ln"
  }

  It "runs js"
    When run node temp.js
    The output should eq "Cloning into './dependencies/alantech/hellodep'..."
    Assert has_dependencies
    Assert has_alantech
    Assert has_hellodep
    Assert has_index
  End

  It "runs agc"
    When run alan-runtime run temp.agc
    The output should eq "Cloning into './dependencies/alantech/hellodep'..."
    Assert has_dependencies
    Assert has_alantech
    Assert has_hellodep
    Assert has_index
  End
End