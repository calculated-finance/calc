export type AssetInfo =
  | {
      native_token: {
        denom: string;
      };
    }
  | {
      token: {
        contract_addr: string;
      };
    };

export const denomFromAsset = (asset: AssetInfo) =>
  'native_token' in asset ? asset.native_token.denom : asset.token.contract_addr;

export const ALTER = {
  native_token: {
    denom: 'ibc/E070901F36B129933202BEB3EB40A78BE242D8ECBA2D1AF9161DF06F35783900',
  },
};

export const ARCH = {
  native_token: {
    denom: 'aarch',
  },
};

export const xARCH = {
  token: {
    contract_addr: 'archway1cutfh7m87cyq5qgqqw49f289qha7vhsg6wtr6rl5fvm28ulnl9ssg0vk0n',
  },
};

export const ampARCH = {
  token: {
    contract_addr: 'archway1fwurjg7ah4v7hhs6xsc3wutqpvmahrfhns285s0lt34tgfdhplxq6m8xg5',
  },
};

export const AKT = {
  native_token: {
    denom: 'ibc/C2CFB1C37C146CF95B0784FD518F8030FEFC76C5800105B1742FB65FFE65F873',
  },
};

export const xAKT = {
  token: {
    contract_addr: 'archway1tl8l2gt9dncdu6huds39dsg366ctllvtnm078qkkad2mnv28erss98tl2n',
  },
};

export const ANDR = {
  native_token: {
    denom: 'ibc/55D94A32095A766971637425D998AAABF8357A1ABCB1CAC8614887BE51BF1FB1',
  },
};

export const ATOM = {
  native_token: {
    denom: 'ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2',
  },
};

export const xATOM = {
  token: {
    contract_addr: 'archway1m273xq2fjmn993jm4kft5c49w2c70yfv5zypt3d92cqp4n5faefqqkuf0l',
  },
};

export const AXV = {
  token: {
    contract_addr: 'archway1ecjefhcf8r60wtfnhwefrxhj9caeqa90fj58cqsaafqveawn6cjs5znd2n',
  },
};

export const BLD = {
  native_token: {
    denom: 'ibc/8CB56C813A5C2387140BBEAABCCE797AFA0960C8D07B171F71A5188726CFED2C',
  },
};

export const xBLD = {
  token: {
    contract_addr: 'archway1yv8uhe795xs4fwz6mjm278yr35ps0yagjchfp39q5x49dty9jgssm5tnkv',
  },
};

export const DEC = {
  native_token: {
    denom: 'ibc/E3409E92F78AE5BF44DBC7C4741901E21EF73B7B8F98C4D48F2BD360AF242C00',
  },
};

export const xDEC = {
  token: {
    contract_addr: 'archway1veyq07az0d7mlp49sa9f9ef56w0dd240vjsy76yv0m4pl5a2x2uq698cs7',
  },
};

export const GRAV = {
  native_token: {
    denom: 'ibc/31D711D31CD5D83D98E76B1486EEDA1A38CD1F7D6FCBD03521FE51323115AECA',
  },
};

export const xGRAV = {
  token: {
    contract_addr: 'archway1zfnzv39cp4dv3jjy0aptn5msc02tjmy602l46u90dt729q80939qjgqcdj',
  },
};

export const IST = {
  native_token: {
    denom: 'ibc/C0336ECF2DF64E7D2C98B1422EC2B38DE9EF33C34AAADF18C6F2E3FFC7BE3615',
  },
};

export const JKL = {
  native_token: {
    denom: 'ibc/926432AE1C5FA4F857B36D970BE7774C7472079506820B857B75C5DE041DD7A3',
  },
};

export const xJKL = {
  token: {
    contract_addr: 'archway1yjdgfut7jkq5xwzyp6p5hs7hdkmszn34zkhun6mglu3falq3yh8sdkaj7j',
  },
};

export const LVN = {
  native_token: {
    denom: 'ibc/6A9571DE6A3F60D7703C3290E2944E806C15A47C1EA6D4AFCD3AE4DC8AF080B1',
  },
};

export const MPWR = {
  native_token: {
    denom: 'ibc/28A2923B26BD4CED9D664B032904D37AABE1F08E8C9E97B0FA18E885CA978EBC',
  },
};

export const xMPWR = {
  token: {
    contract_addr: 'archway1tvrrctwllg8aalc4ruk6a4zxtel8ff7ggxljvu6ffj3wpm2zp8kqyecxpr',
  },
};

export const PLQ = {
  native_token: {
    denom: 'ibc/CFD58F8A64F93940D00CABE85B05A6D0FBA1FF4DF42D3C1E23C06DF30A2BAE1F',
  },
};

export const xPLQ = {
  token: {
    contract_addr: 'archway1h7vfp6hjjluw8n6m2v4tkfdw3getkwqldu59xghltdskt3rh6shqczumjc',
  },
};

export const ROCK = {
  native_token: {
    denom: 'ibc/8F6360B49F40DA2B86F7F1A3335490E126E4DD9BAC60B5ED2EEA08D8A10DC372',
  },
};

export const axlUSDC = {
  native_token: {
    denom: 'ibc/B9E4FD154C92D3A23BEA029906C4C5FF2FE74CB7E3A058290B77197A263CF88B',
  },
};

export const USDC = {
  native_token: {
    denom: 'ibc/43897B9739BD63E3A08A88191999C632E052724AB96BD4C74AE31375C991F48D',
  },
};

export const axlBTC = {
  native_token: {
    denom: 'ibc/3A2DEEBCD51D0B74FE7CE058D40B0BF4C0E556CE9219E8F25F92CF288FF35F56',
  },
};

export const sARCH = {
  token: {
    contract_addr: 'archway1t2llqsvwwunf98v692nqd5juudcmmlu3zk55utx7xtfvznel030saclvq6',
  },
};

export const bnUSD = {
  token: {
    contract_addr: 'archway1l3m84nf7xagkdrcced2y0g367xphnea5uqc3mww3f83eh6h38nqqxnsxz7',
  },
};
