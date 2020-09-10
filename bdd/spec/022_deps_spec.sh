Include build_tools.sh

Describe "@std/deps"
  before() {
    sourceToAll "
      from @std/deps import install, add, commit

      on install {
        add('https://github.com/alantech/hellodep')
        commit()
      }
    "
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

  run_js() {
    node temp.js | head -1
  }

  run_agc() {
    alan run temp.agc | head -1
  }

  It "runs js"
    When run run_js
    The output should eq "Cloning into './dependencies/alantech/hellodep'..."
    Assert has_dependencies
    Assert has_alantech
    Assert has_hellodep
    Assert has_index
  End

  It "runs agc"
    When run run_agc
    The output should eq "Cloning into './dependencies/alantech/hellodep'..."
    Assert has_dependencies
    Assert has_alantech
    Assert has_hellodep
    Assert has_index
  End
End