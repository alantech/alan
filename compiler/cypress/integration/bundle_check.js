describe('Alan Compiler Browser Bundle', () => {
  beforeAll(() => {
    cy.exec('yarn run test-server')
  })

  it('has a "require" global', () => {
    cy.visit('http://localhost:8080/test')
    cy.window().should('have.property', 'require')
  })

  it('can load the "alanCompiler"', () => {
    cy.visit('http://localhost:8080/test')
    cy.window().then((win) => {
      const alanCompiler = win.require('alan-compiler')
      alanCompiler.should.be.a('function')
      win.alanCompiler = alanCompiler
    })
  })
})
