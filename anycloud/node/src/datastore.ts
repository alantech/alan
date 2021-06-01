import { dsdel, dsgetv, dshas, dssetv } from 'alan-js-runtime';

const ns = 'kv';

export const DS = new Proxy({}, {
  get: function (_, dsKey) {
    const dsValue = dsgetv(ns, dsKey);
    return dsValue.length === 2 ? dsValue[1] : 'Value not found';
  },
  set: function (_, dsKey, dsValue) {
    if (dshas(ns, dsKey)) { return false; }
    dssetv(ns, dsKey, dsValue)
    return true;
  },
  deleteProperty: function (_, dsKey) {
    if (!(dshas(ns, dsKey))) { return false; }
    return dsdel(ns, dsKey);
  },
  has: function (_, dsKey) {
    return dshas(ns, dsKey);
  },
});


console.log(DS['foo'] = 'bar');
console.log(DS['foo']);
console.log('foo' in DS);
console.log(delete DS['foo']);
console.log('foo' in DS);
console.log(DS['foo'] = {foo1: "bar1"});
console.log(DS['foo']);
console.log(delete DS['foo']);
