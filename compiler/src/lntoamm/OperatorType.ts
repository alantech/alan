import Type from './Type'

class OperatorType {
  operatorname: string | null
  isPrefix: boolean
  args: Array<Type>
  returnType: Type

  constructor(
    operatorname: string,
    isPrefix: boolean = false,
    args: Array<Type>,
    returnType: Type
  ) {
    this.operatorname = operatorname
    this.isPrefix = isPrefix
    this.args = args
    this.returnType = returnType
  }
}

export default OperatorType