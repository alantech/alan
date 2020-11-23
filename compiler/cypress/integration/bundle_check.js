describe('Alan Compiler Browser Bundle', () => {
  before(() => {
    cy.exec('yarn run test-server')
  })

  after(() => {
    cy.exec('yarn run stop-test-server', { failOnNonZeroExit: false, })
  })

  it('has a "require" global', () => {
    cy.visit('http://localhost:8080/test')
    cy.window().then((win) => {
      cy.log(JSON.stringify(Object.keys(win.document)))
    })
    cy.window().should('have.property', 'require')
  })

  it('can load the "alanCompiler"', () => {
    cy.visit('http://localhost:8080/test')
    cy.window().then((win) => {
      const alanCompiler = win.require('alan-compiler')
      expect(alanCompiler).to.be.a('function')
      win.alanCompiler = alanCompiler
    })
  })

  const helloWorldLn = `
    from @std/app import start, print, exit

    on start {
      print('Hello, World!');
      emit exit 0;
    }
  `

  it('can compile an "ln" file to "amm", "aga", and "js" and execute the "js" correctly', () => {
    cy.visit('http://localhost:8080/test')
    cy.window().then((win) => {
      const alanCompiler = win.require('alan-compiler')
      const amm = alanCompiler('ln', 'amm', helloWorldLn)
      expect(amm).to.be.a('string')
      cy.log(amm)
      const aga = alanCompiler('ln', 'aga', helloWorldLn)
      expect(aga).to.be.a('string')
      cy.log(aga)
      const aga2 = alanCompiler('amm', 'aga', amm)
      expect(aga2).to.be.a('string')
      cy.log(aga2)
      const js = alanCompiler('ln', 'js', helloWorldLn)
      expect(js).to.be.a('string')
      expect(() => {
        eval(js)
      }).to.not.throw(Error)
      const js2 = alanCompiler('amm', 'js', amm)
      expect(js2).to.be.a('string')
      expect(() => {
        eval(js2)
      }).to.not.throw(Error)
    })
  })
})
