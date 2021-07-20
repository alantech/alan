import { v4 as uuid } from 'uuid';
import Fn from './Fn';
import Operator from './Operator';

export const genName = () => '_' + uuid().replace(/-/g, '_');

export const isFnArray = (val: any): val is Array<Fn> => {
  return val instanceof Array && (val.length === 0 || val[0] instanceof Fn);
};

export const isOpArray = (val: any): val is Array<Operator> => {
  return (
    val instanceof Array && (val.length === 0 || val[0] instanceof Operator)
  );
};

/**
 * Assumes valOrMsg is a message if more than 1 argument passed. TS should prevent that though
 * @param valOrMsg the value to debug print or the message to print alongside other values
 * @param vals the values to print after a message
 * @returns the first value passed
 */
export const DBG = function <T>(valOrMsg: T | string, ...vals: T[]): T {
  if (vals.length > 0) {
    console.log('->', valOrMsg, ...vals);
    return vals[0];
  } else {
    console.log('~~', valOrMsg);
    return valOrMsg as T;
  }
};

export const TODO = (task?: string) => {
  throw new Error(`TODO${task !== undefined ? ': ' + task : ''}`);
};

export interface Equalable {
  eq(other: Equalable): boolean;
}

// TODO: is this necessary?
// export class MapButBetter<K extends Equalable, V> {
//   private __keys: K[]
//   private __vals: V[]
//   get size(): number {
//     return this.__keys.length;
//   }
//   constructor() {
//     this.__keys = [];
//     this.__vals = [];
//   }
//   private idxFor(key: K): number {
//     for (let ii = 0; ii < this.__keys.length; ii++) {
//       if (Object.is(this.__keys[ii], key)) {
//         return ii;
//       }
//     }
//     return -1;
//   }
//   clear() {
//     this.__keys = [];
//     this.__vals = [];
//   }
//   /**
//    * @param key The key to delete from the Map
//    * @returns The value that was removed or null if the key is not in the Map
//    */
//   delete(key: K): V {
//     const idx = this.idxFor(key);
//     if (idx === -1) {
//       return null;
//     }
//     const res = this.__vals[idx];
//     this.__keys = this.__keys.splice(idx, 1);
//     this.__vals = this.__vals.splice(idx, 1);
//     return res;
//   }
//   /**
//    * @param key The key of the value to return
//    * @returns The value corresponding to the key or null if the key is not in the Map
//    */
//   get(key: K): V {
//     const idx = this.idxFor(key);
//     if (idx === -1) {
//       return null;
//     }
//     return this.__vals[idx];
//   }
//   /**
//    * @param key The key to look for
//    * @returns True if the key is in the Map
//    */
//   has(key: K): boolean {
//     return this.idxFor(key) !== -1;
//   }
//   /**
//    * @param key The key to set or reassign
//    * @param val The value to assign to the key
//    * @returns The value previously assigned to the key, or null if the key was not previously set
//    */
//   set(key: K, val: V): V {
//     const idx = this.idxFor(key);
//     if (idx === -1) {
//       this.__keys.push(key);
//       this.__vals.push(val);
//       return null;
//     }
//     const res = this.__vals[idx];
//     this.__vals[idx] = val;
//     return res;
//   }
//   // TODO: these methods usually return iterators/generators, but
//   // we don't need to worry about the difference right now...
//   keys(): K[] {
//     return [...this.__keys];
//   }
//   values(): V[] {
//     return [...this.__vals];
//   }
//   entries(): [K, V][] {
//     let res = [];
//     for (let ii = 0; ii < this.size; ii++) {
//       res.push([this.__keys[ii], this.__vals[ii]]);
//     }
//     return res;
//   }
//   forEach(callbackFn: (key: K, val: V) => void) {
//     for (let ii = 0; ii < this.size; ii++) {
//       callbackFn(this.__keys[ii], this.__vals[ii]);
//     }
//   }
// }
