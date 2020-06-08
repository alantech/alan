import * as ammtoaga from '../ammtoaga'
import { agaTextToAgc, } from '../agatoagc'

export const ammToAgc = (filename: string): Buffer => agaTextToAgc(ammtoaga(filename))
export const ammTextToAgc = (str: string): Buffer => agaTextToAgc(ammtoaga.ammTextToAga(str))
