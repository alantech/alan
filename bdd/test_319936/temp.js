const r = require('alan-js-runtime')
const _ef9d7608_f58a_48df_8e4f_d6f6b5971cb0 = 100n
const _1b001efc_f069_4274_b5e9_c4d2c843e05a = true
const _45021775_58d7_430e_8ca8_3218fec9298e = 2n
const _5cd369fa_6e97_439f_b048_6aafef8e0caa = 1n
const _6cd3a32c_327c_4c76_8dc8_03b74fdc446d = 8n
const _b891179f_71ec_41f2_8742_4894ddb1bb38 = false
const _becd17ec_7de6_4fd5_954a_d5044bc5cf5a = 0n
const _b952bb37_cb4d_49e8_946e_ba15dbb11331 = "\n"
r.on('_start', async () => {
    const _cefa5acd_0a0c_4e96_bcd8_90a58ee868af = r.newseq(_ef9d7608_f58a_48df_8e4f_d6f6b5971cb0)
    const _4ed49d72_ad5c_4cab_871c_a960a4fbf22e = async (self, i) => {
        let _34fbc6e5_abd9_40ea_b69e_7444bd3bc2fc = r.zeroed()
        let _5e081546_6f54_4ae9_9dce_be79d8aaa0c2 = r.copybool(_1b001efc_f069_4274_b5e9_c4d2c843e05a)
        const _235307f6_d72c_4f04_b1f6_a063c0169bb2 = r.lti64(i, _45021775_58d7_430e_8ca8_3218fec9298e)
        const _e43b13ec_6852_4b30_a024_e5b78b40abdb = async () => {
            const _e5a283e6_475d_4208_8390_b9ef4592d7e2 = r.okR(_5cd369fa_6e97_439f_b048_6aafef8e0caa, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
            _34fbc6e5_abd9_40ea_b69e_7444bd3bc2fc = r.refv(_e5a283e6_475d_4208_8390_b9ef4592d7e2)
            _5e081546_6f54_4ae9_9dce_be79d8aaa0c2 = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
          }
        const _38919ccf_23e7_4e49_ae15_fbb1fd24f4a2 = await r.condfn(_235307f6_d72c_4f04_b1f6_a063c0169bb2, _e43b13ec_6852_4b30_a024_e5b78b40abdb)
        const _ab8eba41_58c2_41a4_bd12_6bf31b6ca0cd = async () => {
            const _97dd680e_e724_47a8_a832_ddd2994c14a4 = r.notbool(_235307f6_d72c_4f04_b1f6_a063c0169bb2)
            const _b4cf414c_40d7_4255_b797_eb7e624b865a = async () => {
                let _efc49224_41d0_4b97_8e9a_d2c583a03958 = r.zeroed()
                let _a12f6330_76de_4c84_bad4_4d13f465c2fd = r.copybool(_1b001efc_f069_4274_b5e9_c4d2c843e05a)
                const _2cdd3260_ec66_4824_8a8c_5b803216eeb3 = r.okR(i, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
                const _f01633fc_d26b_4e81_9ab6_9ae5c63dc45a = r.okR(_5cd369fa_6e97_439f_b048_6aafef8e0caa, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
                const _e4361bfc_383c_4feb_8eb2_72c8cfec9ea0 = r.subi64(_2cdd3260_ec66_4824_8a8c_5b803216eeb3, _f01633fc_d26b_4e81_9ab6_9ae5c63dc45a)
                const _941a24a8_23a1_4497_b378_672e68ed8f21 = r.getOrR(_e4361bfc_383c_4feb_8eb2_72c8cfec9ea0, _becd17ec_7de6_4fd5_954a_d5044bc5cf5a)
                const _638086ee_c1b8_4430_ab0c_a94efe0b585c = await r.selfrec(self, _941a24a8_23a1_4497_b378_672e68ed8f21)
                const _783c0117_480d_4e69_9ede_fbc3ee5c27c5 = r.okR(i, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
                const _2a520b81_267d_4910_8db9_9ef0bc5c8a66 = r.okR(_45021775_58d7_430e_8ca8_3218fec9298e, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
                const _bfb4e7a7_031b_4e0c_a55a_9d8f1398f2c2 = r.subi64(_783c0117_480d_4e69_9ede_fbc3ee5c27c5, _2a520b81_267d_4910_8db9_9ef0bc5c8a66)
                const _be439746_1c06_4525_9c47_26a766b9b36e = r.getOrR(_bfb4e7a7_031b_4e0c_a55a_9d8f1398f2c2, _becd17ec_7de6_4fd5_954a_d5044bc5cf5a)
                const _682c427f_13b8_4dad_a5b3_b4d83ff4c0c1 = await r.selfrec(self, _be439746_1c06_4525_9c47_26a766b9b36e)
                const _2f9c1d95_3003_4688_aa29_c7cc2d377280 = r.isErr(_638086ee_c1b8_4430_ab0c_a94efe0b585c)
                const _784f25e2_ccde_4b57_8f23_b7482fd0f254 = async () => {
                    _efc49224_41d0_4b97_8e9a_d2c583a03958 = r.refv(_638086ee_c1b8_4430_ab0c_a94efe0b585c)
                    _a12f6330_76de_4c84_bad4_4d13f465c2fd = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
                  }
                const _fbb4a520_5068_4272_b7d5_4f2082b4cdf5 = await r.condfn(_2f9c1d95_3003_4688_aa29_c7cc2d377280, _784f25e2_ccde_4b57_8f23_b7482fd0f254)
                const _3f5f8772_66bc_404f_9d34_b11133e26820 = async () => {
                    const _2a0351c1_47b7_49e6_bc75_aab5d30ea3e2 = r.isErr(_682c427f_13b8_4dad_a5b3_b4d83ff4c0c1)
                    const _66cd6289_6e91_4883_83d2_98153f4b425f = async () => {
                        _efc49224_41d0_4b97_8e9a_d2c583a03958 = r.refv(_682c427f_13b8_4dad_a5b3_b4d83ff4c0c1)
                        _a12f6330_76de_4c84_bad4_4d13f465c2fd = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
                      }
                    const _665ef029_44d4_428e_b067_f0f2e5df86ce = await r.condfn(_2a0351c1_47b7_49e6_bc75_aab5d30ea3e2, _66cd6289_6e91_4883_83d2_98153f4b425f)
                    const _351250f3_96f1_4ca7_830c_1701e33cb1e6 = async () => {
                        const _166b8693_5a68_4897_adb1_2dc30d54ab5f = r.getOrR(_638086ee_c1b8_4430_ab0c_a94efe0b585c, _becd17ec_7de6_4fd5_954a_d5044bc5cf5a)
                        const _ef6d9600_79c3_42d2_a025_b365c1d4ce31 = r.getOrR(_682c427f_13b8_4dad_a5b3_b4d83ff4c0c1, _becd17ec_7de6_4fd5_954a_d5044bc5cf5a)
                        const _71acbbfa_0751_4eea_b83e_f1584f5fc725 = r.okR(_166b8693_5a68_4897_adb1_2dc30d54ab5f, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
                        const _841250dc_e1b6_4856_936d_d08039d1444d = r.okR(_ef6d9600_79c3_42d2_a025_b365c1d4ce31, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
                        const _9646e2ea_633e_450d_b2b0_af1ce00a2285 = r.addi64(_71acbbfa_0751_4eea_b83e_f1584f5fc725, _841250dc_e1b6_4856_936d_d08039d1444d)
                        _efc49224_41d0_4b97_8e9a_d2c583a03958 = r.refv(_9646e2ea_633e_450d_b2b0_af1ce00a2285)
                        _a12f6330_76de_4c84_bad4_4d13f465c2fd = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
                      }
                    const _b16c09b7_d3a3_4151_a805_978b10c5e188 = await r.condfn(_a12f6330_76de_4c84_bad4_4d13f465c2fd, _351250f3_96f1_4ca7_830c_1701e33cb1e6)
                  }
                const _cdca0930_7df9_4404_8c73_25029576a2f3 = await r.condfn(_a12f6330_76de_4c84_bad4_4d13f465c2fd, _3f5f8772_66bc_404f_9d34_b11133e26820)
                _34fbc6e5_abd9_40ea_b69e_7444bd3bc2fc = r.refv(_efc49224_41d0_4b97_8e9a_d2c583a03958)
                _5e081546_6f54_4ae9_9dce_be79d8aaa0c2 = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
              }
            const _df007a3a_a81f_4ffa_a6b8_2becb0ef415f = await r.condfn(_97dd680e_e724_47a8_a832_ddd2994c14a4, _b4cf414c_40d7_4255_b797_eb7e624b865a)
          }
        const _b48b019d_708e_42f7_87ac_1776f0f9d603 = await r.condfn(_5e081546_6f54_4ae9_9dce_be79d8aaa0c2, _ab8eba41_58c2_41a4_bd12_6bf31b6ca0cd)
        return _34fbc6e5_abd9_40ea_b69e_7444bd3bc2fc
      }
    let _08a79e00_bde9_4c87_a607_4f5d893208bd = r.seqrec(_cefa5acd_0a0c_4e96_bcd8_90a58ee868af, _4ed49d72_ad5c_4cab_871c_a960a4fbf22e)
    const _9fdcd30c_50cd_4977_b0c1_ae7d70508c25 = await r.selfrec(_08a79e00_bde9_4c87_a607_4f5d893208bd, _6cd3a32c_327c_4c76_8dc8_03b74fdc446d)
    let _b3d120bd_7bcc_4a4d_b329_d5a9f79b3e96 = r.zeroed()
    let _0536cf0c_d68a_4a19_98ac_fac724271dc0 = r.copybool(_1b001efc_f069_4274_b5e9_c4d2c843e05a)
    const _2d5883c2_7f78_41ba_a874_203e65d82762 = r.isOk(_9fdcd30c_50cd_4977_b0c1_ae7d70508c25)
    const _43738f9e_9516_4242_8e6c_092ed0d8d0bc = async () => {
        const _3209cd92_e7f8_45ab_90d5_488b850a3ce5 = r.getR(_9fdcd30c_50cd_4977_b0c1_ae7d70508c25)
        const _c009974d_4be7_4b6c_8cd7_49615b14b1ba = r.i64str(_3209cd92_e7f8_45ab_90d5_488b850a3ce5)
        _b3d120bd_7bcc_4a4d_b329_d5a9f79b3e96 = r.refv(_c009974d_4be7_4b6c_8cd7_49615b14b1ba)
        _0536cf0c_d68a_4a19_98ac_fac724271dc0 = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
      }
    const _1344db9e_bbbd_4211_b352_1c918f9c6e5d = await r.condfn(_2d5883c2_7f78_41ba_a874_203e65d82762, _43738f9e_9516_4242_8e6c_092ed0d8d0bc)
    const _a52a2798_7113_488d_ad17_8d4a49c11bce = async () => {
        const _9ddc9ed9_84c1_47f2_aa61_f9a03536033b = r.notbool(_2d5883c2_7f78_41ba_a874_203e65d82762)
        const _3c5dddd4_2258_4237_a94e_efc3890af780 = async () => {
            const _6d85118d_413e_4003_a3d7_1dff878c71ef = r.noerr()
            const _99bea54e_b278_4e02_97ba_590adf9f5e15 = r.getErr(_9fdcd30c_50cd_4977_b0c1_ae7d70508c25, _6d85118d_413e_4003_a3d7_1dff878c71ef)
            const _e5d06258_2ddc_49e3_a9ff_b02996ac4d13 = r.errorstr(_99bea54e_b278_4e02_97ba_590adf9f5e15)
            _b3d120bd_7bcc_4a4d_b329_d5a9f79b3e96 = r.refv(_e5d06258_2ddc_49e3_a9ff_b02996ac4d13)
            _0536cf0c_d68a_4a19_98ac_fac724271dc0 = r.copybool(_b891179f_71ec_41f2_8742_4894ddb1bb38)
          }
        const _994e88ba_71c8_4d19_8b7f_63ec77de068c = await r.condfn(_9ddc9ed9_84c1_47f2_aa61_f9a03536033b, _3c5dddd4_2258_4237_a94e_efc3890af780)
      }
    const _c79d1011_1dcd_41b3_954b_f40e3c5c5e2f = await r.condfn(_0536cf0c_d68a_4a19_98ac_fac724271dc0, _a52a2798_7113_488d_ad17_8d4a49c11bce)
    const _4c595a07_f7a3_40a1_bb98_faf3f19dcbad = r.catstr(_b3d120bd_7bcc_4a4d_b329_d5a9f79b3e96, _b952bb37_cb4d_49e8_946e_ba15dbb11331)
    const _b00c943f_da58_43ce_afc3_adb0f1df586f = r.stdoutp(_4c595a07_f7a3_40a1_bb98_faf3f19dcbad)
    r.emit('exit', _becd17ec_7de6_4fd5_954a_d5044bc5cf5a)
  })
r.on('stdout', async (out) => {
    const _a9bd305a_7a8c_4075_a09d_7697b8ac73f1 = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _5d39f8de_e171_4b5b_afa6_5ebe0a2f39b5 = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _2187e3e5_2c1e_4624_bf94_b1c3e4ba5c26 = r.stderrp(err)
  })
r.emit('_start', undefined)
