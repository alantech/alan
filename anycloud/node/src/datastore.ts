export const DS = new Proxy({}, {
  get: function (oTarget, dsKey) {
    return oTarget[dsKey]; //|| /** dsget call */ || undefined;
  },
  set: function (oTarget, dsKey, dsValue) {
    if (dsKey in oTarget) { return false; }
    return oTarget[dsKey] = dsValue/** dsset call */;
  },
  deleteProperty: function (oTarget, dsKey) {
    if (!(dsKey in oTarget)) { return false; }
    return delete oTarget[dsKey]/** dsdet call */;
  },
  has: function (oTarget, dsKey) {
    return dsKey in oTarget // || /** dshas call */;
  },
});
