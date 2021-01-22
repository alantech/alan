const r = require('alan-js-runtime')
const _6727c824_ad55_4a49_9e97_c5470858daf5 = 1n
const _d355064f_0d3a_4ed1_a25d_6c35b81fd9a4 = 2n
const _446737ae_195a_4d87_a962_2ecb53e1e570 = 3n
const _7879cc6b_a225_4a3e_8ce4_3ab867c36d0c = 4n
const _6464c033_fabb_4818_8318_bd72d51ec84f = 5n
const _131a6693_5c3a_4196_a39c_c9800db0cb68 = 8n
const _88bd09ec_af4c_4846_b741_9157867b7c6f = 0n
const _a92a182b_31c8_45dc_aec7_29967a24e87e = ", "
const _51688d75_1be4_4a15_9f59_8d7bf83f5f94 = "\n"
r.on('_start', async () => {
    const _8333a557_d7b2_4a3b_b5ec_91a34ebaa843 = r.newarr(_6464c033_fabb_4818_8318_bd72d51ec84f)
    r.pusharr(_8333a557_d7b2_4a3b_b5ec_91a34ebaa843, _6727c824_ad55_4a49_9e97_c5470858daf5, _131a6693_5c3a_4196_a39c_c9800db0cb68)
    r.pusharr(_8333a557_d7b2_4a3b_b5ec_91a34ebaa843, _d355064f_0d3a_4ed1_a25d_6c35b81fd9a4, _131a6693_5c3a_4196_a39c_c9800db0cb68)
    r.pusharr(_8333a557_d7b2_4a3b_b5ec_91a34ebaa843, _446737ae_195a_4d87_a962_2ecb53e1e570, _131a6693_5c3a_4196_a39c_c9800db0cb68)
    r.pusharr(_8333a557_d7b2_4a3b_b5ec_91a34ebaa843, _7879cc6b_a225_4a3e_8ce4_3ab867c36d0c, _131a6693_5c3a_4196_a39c_c9800db0cb68)
    r.pusharr(_8333a557_d7b2_4a3b_b5ec_91a34ebaa843, _6464c033_fabb_4818_8318_bd72d51ec84f, _131a6693_5c3a_4196_a39c_c9800db0cb68)
    const _da2ead26_49ae_484f_a999_02690305b9e7 = async (x) => {
        const _9183fc02_cec9_4d85_9690_5ffe67f44a02 = r.okR(x, _131a6693_5c3a_4196_a39c_c9800db0cb68)
        const _92c23e96_a6eb_4925_a97a_b38e301dc0f6 = r.okR(_d355064f_0d3a_4ed1_a25d_6c35b81fd9a4, _131a6693_5c3a_4196_a39c_c9800db0cb68)
        const _c8cb23c1_dd95_40ff_8f3c_da720fefc53b = r.muli64(_9183fc02_cec9_4d85_9690_5ffe67f44a02, _92c23e96_a6eb_4925_a97a_b38e301dc0f6)
        const _65dbf0df_6216_4b7e_99b2_1f8f9f752036 = r.getOrR(_c8cb23c1_dd95_40ff_8f3c_da720fefc53b, _88bd09ec_af4c_4846_b741_9157867b7c6f)
        return _65dbf0df_6216_4b7e_99b2_1f8f9f752036
      }
    const _e9677f3e_9ddc_4005_aced_e7c0f46e624c = await r.map(_8333a557_d7b2_4a3b_b5ec_91a34ebaa843, _da2ead26_49ae_484f_a999_02690305b9e7)
    const _307c6037_7fc1_4f1c_9497_3b22f05b17e7 = async (n) => {
        const _967a9584_67ea_40ee_8abf_ba6aeeb2ef5a = r.i64str(n)
        return _967a9584_67ea_40ee_8abf_ba6aeeb2ef5a
      }
    const _a246c3c8_60ac_4b1f_8ad9_8477f7992aad = await r.map(_e9677f3e_9ddc_4005_aced_e7c0f46e624c, _307c6037_7fc1_4f1c_9497_3b22f05b17e7)
    const _581ce1a6_494c_4723_83b4_634de8781c3a = r.join(_a246c3c8_60ac_4b1f_8ad9_8477f7992aad, _a92a182b_31c8_45dc_aec7_29967a24e87e)
    const _61730be5_abcc_4e02_bedc_8ed6ab6715f3 = r.catstr(_581ce1a6_494c_4723_83b4_634de8781c3a, _51688d75_1be4_4a15_9f59_8d7bf83f5f94)
    const _ced58366_2566_4596_9c74_2f3096f3f239 = r.stdoutp(_61730be5_abcc_4e02_bedc_8ed6ab6715f3)
    r.emit('exit', _88bd09ec_af4c_4846_b741_9157867b7c6f)
  })
r.on('stdout', async (out) => {
    const _8ff868d9_9792_48a3_8f16_c925add6dcf1 = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _ad4df22b_7c47_4b88_b7f9_5548c2e63114 = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _d483abca_fab8_4e52_810e_265bbf19e3da = r.stderrp(err)
  })
r.emit('_start', undefined)
