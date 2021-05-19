const r = require('alan-js-runtime')
const _b1c07a5a_89d2_45f3_990d_13f0f5aaa2f2 = "Testing..."
const _3d416540_9d6a_4388_be44_da85aad16362 = "\n"
const _c1594947_8b36_4a6e_bc14_44662f8543c0 = "1,2,3"
const _b78cfe85_74fc_416c_a8d1_c9dd30fad0b4 = ","
const _8a544b0b_fc67_43ab_b2e5_0159c0e42d78 = 0n
const _e3a2a484_e8db_4e3f_80fb_4e7234a52b8c = 8n
const _6d85da21_3a96_4d12_90d1_a7d0e60777c8 = true
const _f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e = false
const _ed3cb037_5104_4a0d_a604_0748630c28d6 = 1n
const _03137bb8_72db_4f03_8772_9375e09417b3 = 2n
r.on('_start', async () => {
    const _c9b08f26_d176_418b_8ce8_ef1cf177d0d1 = r.catstr(_b1c07a5a_89d2_45f3_990d_13f0f5aaa2f2, _3d416540_9d6a_4388_be44_da85aad16362)
    const _5a42b09c_d056_42c9_b5ed_dacb891342f5 = r.stdoutp(_c9b08f26_d176_418b_8ce8_ef1cf177d0d1)
    const _6df1fd25_2474_4299_999e_3d8265d7493d = r.split(_c1594947_8b36_4a6e_bc14_44662f8543c0, _b78cfe85_74fc_416c_a8d1_c9dd30fad0b4)
    const _d066b0cc_c92d_47ce_86ca_3f0001beb4a2 = r.lenarr(_6df1fd25_2474_4299_999e_3d8265d7493d)
    const _ceb9d3f2_cacd_427c_b9fd_2c9abcfb2c34 = r.i64str(_d066b0cc_c92d_47ce_86ca_3f0001beb4a2)
    const _ecf945da_231e_4d27_9f08_5797de408ccd = r.catstr(_ceb9d3f2_cacd_427c_b9fd_2c9abcfb2c34, _3d416540_9d6a_4388_be44_da85aad16362)
    const _b3eb1c38_ed94_40ce_bc1e_694ac62075e2 = r.stdoutp(_ecf945da_231e_4d27_9f08_5797de408ccd)
    const _4380d120_a5d6_4cec_8c6e_d94a43fcc11c = r.okR(_8a544b0b_fc67_43ab_b2e5_0159c0e42d78, _e3a2a484_e8db_4e3f_80fb_4e7234a52b8c)
    const _158abd46_7a52_4be2_bc69_c068a667a734 = r.resfrom(_6df1fd25_2474_4299_999e_3d8265d7493d, _4380d120_a5d6_4cec_8c6e_d94a43fcc11c)
    let _7e7e8a9a_9f65_4e74_9593_788825953c7a = r.zeroed()
    let _ca3e76d0_91e1_4407_bc5b_b8f3fe162125 = r.copybool(_6d85da21_3a96_4d12_90d1_a7d0e60777c8)
    const _71b083eb_6056_42d8_931e_3182168c45f3 = r.isOk(_158abd46_7a52_4be2_bc69_c068a667a734)
    const _a42c42d8_1856_4bfc_95dc_9288627670cb = async () => {
        const _b24f208f_4593_4854_a482_cc40b6765dc8 = r.getR(_158abd46_7a52_4be2_bc69_c068a667a734)
        _7e7e8a9a_9f65_4e74_9593_788825953c7a = r.refv(_b24f208f_4593_4854_a482_cc40b6765dc8)
        _ca3e76d0_91e1_4407_bc5b_b8f3fe162125 = r.copybool(_f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e)
      }
    const _b25640b6_573d_43e0_9025_8c8a5bf753c1 = await r.condfn(_71b083eb_6056_42d8_931e_3182168c45f3, _a42c42d8_1856_4bfc_95dc_9288627670cb)
    const _6f05443a_0e74_437d_b34a_ce2358e8927d = async () => {
        const _dd77a807_5d3a_44a1_95ae_a126177f1607 = r.notbool(_71b083eb_6056_42d8_931e_3182168c45f3)
        const _a349b4cf_9dfe_4cc9_9330_491d43ae3e47 = async () => {
            const _3d28ce41_175a_4579_81d4_3a0701922664 = r.noerr()
            const _a591459b_3e50_47c3_ad62_63861891f7e0 = r.getErr(_158abd46_7a52_4be2_bc69_c068a667a734, _3d28ce41_175a_4579_81d4_3a0701922664)
            const _c8de631f_aee0_489a_9911_529c09b4b922 = r.errorstr(_a591459b_3e50_47c3_ad62_63861891f7e0)
            _7e7e8a9a_9f65_4e74_9593_788825953c7a = r.refv(_c8de631f_aee0_489a_9911_529c09b4b922)
            _ca3e76d0_91e1_4407_bc5b_b8f3fe162125 = r.copybool(_f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e)
          }
        const _f1eb2365_cced_4aba_8f84_ff936797b352 = await r.condfn(_dd77a807_5d3a_44a1_95ae_a126177f1607, _a349b4cf_9dfe_4cc9_9330_491d43ae3e47)
      }
    const _e26a712a_13b9_41bd_9847_9f04ecfdf2a8 = await r.condfn(_ca3e76d0_91e1_4407_bc5b_b8f3fe162125, _6f05443a_0e74_437d_b34a_ce2358e8927d)
    const _69e69e34_d2ea_4494_a752_4c40fea5943f = r.catstr(_7e7e8a9a_9f65_4e74_9593_788825953c7a, _3d416540_9d6a_4388_be44_da85aad16362)
    const _0b7b4685_036c_4a5b_9302_1e5e74a8b46c = r.stdoutp(_69e69e34_d2ea_4494_a752_4c40fea5943f)
    const _021c2d16_8850_40bc_8aad_648d18a85d6c = r.okR(_ed3cb037_5104_4a0d_a604_0748630c28d6, _e3a2a484_e8db_4e3f_80fb_4e7234a52b8c)
    const _8d06b5cd_3cf4_4799_aa2d_0169e6367e4e = r.resfrom(_6df1fd25_2474_4299_999e_3d8265d7493d, _021c2d16_8850_40bc_8aad_648d18a85d6c)
    let _08a9ecba_5d43_437a_83fb_bb2b26890104 = r.zeroed()
    let _9ae41b3e_5a53_4df9_8e25_4af8062e0779 = r.copybool(_6d85da21_3a96_4d12_90d1_a7d0e60777c8)
    const _481d1d58_307a_4945_9021_1209c78b00cf = r.isOk(_8d06b5cd_3cf4_4799_aa2d_0169e6367e4e)
    const _d4dc2c84_a486_4803_a7d4_ec7a6ed71d4b = async () => {
        const _51b5e577_702a_4ee5_a652_9cf07294d461 = r.getR(_8d06b5cd_3cf4_4799_aa2d_0169e6367e4e)
        _08a9ecba_5d43_437a_83fb_bb2b26890104 = r.refv(_51b5e577_702a_4ee5_a652_9cf07294d461)
        _9ae41b3e_5a53_4df9_8e25_4af8062e0779 = r.copybool(_f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e)
      }
    const _5d9c1405_49d7_4760_923d_4c9dec93b3d8 = await r.condfn(_481d1d58_307a_4945_9021_1209c78b00cf, _d4dc2c84_a486_4803_a7d4_ec7a6ed71d4b)
    const _efd75432_855d_4cdc_8b89_2421ac01364e = async () => {
        const _d4cb6ab8_3e3b_46b5_890c_8a55f8048aac = r.notbool(_481d1d58_307a_4945_9021_1209c78b00cf)
        const _980b78c7_7054_423e_9a47_0560c56d26be = async () => {
            const _1dc51f6f_c8f4_48cd_97e7_5b10de966607 = r.noerr()
            const _27b928a1_e44a_43c7_bc6f_6388c9959443 = r.getErr(_8d06b5cd_3cf4_4799_aa2d_0169e6367e4e, _1dc51f6f_c8f4_48cd_97e7_5b10de966607)
            const _50966f5e_3062_4931_9c6b_357fa568fe2e = r.errorstr(_27b928a1_e44a_43c7_bc6f_6388c9959443)
            _08a9ecba_5d43_437a_83fb_bb2b26890104 = r.refv(_50966f5e_3062_4931_9c6b_357fa568fe2e)
            _9ae41b3e_5a53_4df9_8e25_4af8062e0779 = r.copybool(_f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e)
          }
        const _cb1e32cb_acaa_4265_a0fa_0c8934856505 = await r.condfn(_d4cb6ab8_3e3b_46b5_890c_8a55f8048aac, _980b78c7_7054_423e_9a47_0560c56d26be)
      }
    const _289ef8f9_42b9_4931_8d98_aa9615e99763 = await r.condfn(_9ae41b3e_5a53_4df9_8e25_4af8062e0779, _efd75432_855d_4cdc_8b89_2421ac01364e)
    const _429acc2c_a747_4aef_843c_d7c499279757 = r.catstr(_08a9ecba_5d43_437a_83fb_bb2b26890104, _3d416540_9d6a_4388_be44_da85aad16362)
    const _4e488305_2825_4828_9e42_e6637484765f = r.stdoutp(_429acc2c_a747_4aef_843c_d7c499279757)
    const _6719360f_3cea_4fea_8e03_6ade82beacf3 = r.okR(_03137bb8_72db_4f03_8772_9375e09417b3, _e3a2a484_e8db_4e3f_80fb_4e7234a52b8c)
    const _8013b221_d331_4694_98e6_20d5341d9b0d = r.resfrom(_6df1fd25_2474_4299_999e_3d8265d7493d, _6719360f_3cea_4fea_8e03_6ade82beacf3)
    let _566fa2ae_3f80_43d2_aa72_dea16c632354 = r.zeroed()
    let _718f757b_3a4e_4022_9d2b_94a93936895b = r.copybool(_6d85da21_3a96_4d12_90d1_a7d0e60777c8)
    const _3f017eec_8173_4b77_9785_913478901cec = r.isOk(_8013b221_d331_4694_98e6_20d5341d9b0d)
    const _8dd27cb7_b0cd_4400_aed7_16f15edf7abf = async () => {
        const _a873f834_56e5_442c_9edb_364e7b796206 = r.getR(_8013b221_d331_4694_98e6_20d5341d9b0d)
        _566fa2ae_3f80_43d2_aa72_dea16c632354 = r.refv(_a873f834_56e5_442c_9edb_364e7b796206)
        _718f757b_3a4e_4022_9d2b_94a93936895b = r.copybool(_f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e)
      }
    const _a5b3843a_971f_4eb8_868c_662e222d3653 = await r.condfn(_3f017eec_8173_4b77_9785_913478901cec, _8dd27cb7_b0cd_4400_aed7_16f15edf7abf)
    const _128df0cd_3be9_4327_b24c_db7faeb0469d = async () => {
        const _d1b5e5ae_c40b_458c_821b_cf5ea6218dd1 = r.notbool(_3f017eec_8173_4b77_9785_913478901cec)
        const _a1cc5973_66f6_4a72_aa47_f5643c94e9b9 = async () => {
            const _3ec69b6e_31c8_48c8_91de_6bd6783d2afc = r.noerr()
            const _b935d26c_a14a_4b29_b31b_212144ddeff0 = r.getErr(_8013b221_d331_4694_98e6_20d5341d9b0d, _3ec69b6e_31c8_48c8_91de_6bd6783d2afc)
            const _c69047cd_f81b_47f3_96de_38150c133485 = r.errorstr(_b935d26c_a14a_4b29_b31b_212144ddeff0)
            _566fa2ae_3f80_43d2_aa72_dea16c632354 = r.refv(_c69047cd_f81b_47f3_96de_38150c133485)
            _718f757b_3a4e_4022_9d2b_94a93936895b = r.copybool(_f4cd39d5_ef87_4dbb_87f9_e06fa8ea8a9e)
          }
        const _66f9fdf7_bc80_4f47_ac60_8abda6b4d6c1 = await r.condfn(_d1b5e5ae_c40b_458c_821b_cf5ea6218dd1, _a1cc5973_66f6_4a72_aa47_f5643c94e9b9)
      }
    const _49ef0cb7_26f1_413a_bbba_4abc9ad21d3d = await r.condfn(_718f757b_3a4e_4022_9d2b_94a93936895b, _128df0cd_3be9_4327_b24c_db7faeb0469d)
    const _20dd4542_b11b_4bc5_a8cf_f808c5571089 = r.catstr(_566fa2ae_3f80_43d2_aa72_dea16c632354, _3d416540_9d6a_4388_be44_da85aad16362)
    const _331a4da2_ebc0_425d_a5b6_4656b1871aa2 = r.stdoutp(_20dd4542_b11b_4bc5_a8cf_f808c5571089)
    r.emit('exit', _8a544b0b_fc67_43ab_b2e5_0159c0e42d78)
  })
r.on('stdout', async (out) => {
    const _ba7e5381_8653_43cd_88ad_c45d651b402f = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _cee0a253_00bb_4441_b078_8bdc589dd1d0 = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _85602f56_5330_498c_8faa_fd5f39dd7e31 = r.stderrp(err)
  })
r.emit('_start', undefined)
