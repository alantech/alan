import Type from './Type'

class FunctionType {
  functionname: string | null
  args: Array<Type>
  returnType: Type

  constructor(...args: Array<any>) {
    if (args.length === 1) {
      this.functionname = null
      this.args = []
      this.returnType = args[0]
    } else if (args.length === 2) {
      if (typeof args[0] === "string") {
        this.functionname = args[0]
        this.args = []
        this.returnType = args[1]
      } else if (args[0] instanceof Array) {
        this.functionname = null
        this.args = args[0]
        this.returnType = args[1]
      }
    } else if (args.length === 3) {
      this.functionname = args[0]
      this.args = args[1]
      this.returnType = args[2]
    }
  }
}

export default FunctionType
