import { sort } from 'ramda';
import {
  AKT,
  ALTER,
  ANDR,
  ARCH,
  ATOM,
  AXV,
  AssetInfo,
  BLD,
  DEC,
  GRAV,
  IST,
  JKL,
  LVN,
  MPWR,
  PLQ,
  ROCK,
  USDC,
  axlBTC,
  axlUSDC,
  bnUSD,
  denomFromAsset,
  sARCH,
  xAKT,
  xARCH,
  xATOM,
  xBLD,
  xDEC,
  xGRAV,
  xJKL,
  xMPWR,
  xPLQ,
} from './denoms';

export const keyFromAssets = (a: AssetInfo, b: AssetInfo) =>
  sort((a, b) => (a > b ? 1 : -1), [denomFromAsset(a), denomFromAsset(b)]).join('-');

export const POOLS = {
  [keyFromAssets(xARCH, ALTER)]: {
    address: 'archway1vwg8yxwm0dfdjwg7txv5k7rtu04vjza0rug9mxkr07nxw9m2caese79trq',
    pool_type: 'standard',
  },
  [keyFromAssets(ARCH, xARCH)]: {
    address: 'archway1vq9jza8kuz80f7ypyvm3pttvpcwlsa5fvum9hxhew5u95mffknxsjy297r',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, xJKL)]: {
    address: 'archway1jt05s9ywp7cmdupfwvfkun8sjekegepkde8fgd9pkyqzx73nqewqvj2847',
    pool_type: 'standard',
  },
  [keyFromAssets(xJKL, JKL)]: {
    address: 'archway1hyww8hnnl9jafeau68l9dwruty0xskp3v9ukdy5jzeg633x2qtasp6dl40',
    pool_type: 'stable',
  },
  [keyFromAssets(xJKL, xAKT)]: {
    address: 'archway139hgd4rm3xyuqyrn63ardjxkg7puzafne7u3pj04qag7ld9cyhnqk9540y',
    pool_type: 'standard',
  },
  [keyFromAssets(xAKT, AKT)]: {
    address: 'archway1lym737d6zvckgw56pu85j2jachd7e6y2e67ydc9sky9rl6a6kpgs2la2f3',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, ANDR)]: {
    address: 'archway1au4l3hpu0uhgyvkjrtmk2f6k2lkntp89z4t3m30570vpq3clgujsp2hj7v',
    pool_type: 'standard',
  },
  [keyFromAssets(ATOM, xATOM)]: {
    address: 'archway14stujwyxsqddhe4wwcxx4ykkqp585lzems9rhzxyuh5k87lta8rqmsa7jz',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, xATOM)]: {
    address: 'archway1tadvtdm4ah6vnt8tzhfmk0e3aj62wxsvrq43zxlerqspufs2ydjs88pspp',
    pool_type: 'standard',
  },
  [keyFromAssets(xARCH, AXV)]: {
    address: 'archway1d5s2ynrfnnjckg25h57693s5c9yljesqny2f4jcgycq6d7l7hvks08hfuv',
    pool_type: 'standard',
  },
  [keyFromAssets(xARCH, xBLD)]: {
    address: 'archway1yr9nad5dv34nze2wtx6ye5hl38ymy5q70t4eu96l8hydwhs9tnkq3gwmev',
    pool_type: 'standard',
  },
  [keyFromAssets(xBLD, BLD)]: {
    address: 'archway1yczh6rtqh090fp66k74a5stuj548t83ncpsw2qy6v8e02thhlelsau720h',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, xDEC)]: {
    address: 'archway1k6falt0jp8qjfycvh2fgqlwl2znxdqql72knvrljtd3ezh5cmqesfxmzdt',
    pool_type: 'standard',
  },
  [keyFromAssets(xDEC, DEC)]: {
    address: 'archway1h6gxz86dtx34087lp5psdnc4glhsnhqyq3fym3ddt2s9awllexhqs9hpzx',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, xGRAV)]: {
    address: 'archway1ure4evg4krw593f8fh8wh2ny4d33gat9rga0nwpj7sq9xhvl4xgq0law66',
    pool_type: 'standard',
  },
  [keyFromAssets(xGRAV, GRAV)]: {
    address: 'archway1qdpytzeej5q8yn3t228t90xh72s52pht98jx0yhpnlzhtn2s63kql5la8n',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, IST)]: {
    address: 'archway18wdt3zsqgk2rgvgkqga44z5hz4p2yjsuszxpgeew8ftv4nev99esar8wt2',
    pool_type: 'standard',
  },
  [keyFromAssets(xARCH, LVN)]: {
    address: 'archway1a0klnfegndslt3j3pjt84uqyc3ydq8swz0cq32v9lmjem4q5a6psjsdcq3',
    pool_type: 'standard',
  },
  [keyFromAssets(xARCH, xMPWR)]: {
    address: 'archway19afhcz9s5cr98um84qjz0vu6khqpduurhep9u3ds3lytp7c5m2mstgs23d',
    pool_type: 'standard',
  },
  [keyFromAssets(xMPWR, MPWR)]: {
    address: 'archway1dwha2307pwm98cah4jkwahf86cjd7qd0gpufp7mggefkrr5gch5q96wd56',
    pool_type: 'stable',
  },
  [keyFromAssets(xARCH, xPLQ)]: {
    address: 'archway1qcm2umkgxvtuqept3mtess0wy57n6q6ejh42c7d7wp3q80advs8q0e0nz3',
    pool_type: 'standard',
  },
  [keyFromAssets(xPLQ, PLQ)]: {
    address: 'archway172h6yuxa2n97a44ew7gkur84dkram69wez8xfsd30ruuq7ah6p5q6fnkvc',
    pool_type: 'stable',
  },
  [keyFromAssets(xPLQ, AXV)]: {
    address: 'archway1ncyl7mqukgzxwvhe7pl50trc99gcwaqqdc0a80tarspqtqjdx2kq63u6vs',
    pool_type: 'standard',
  },
  [keyFromAssets(xARCH, ROCK)]: {
    address: 'archway19cqd9ats5azjw4emlh5e0ucc8xn9ct37efk9pt2hlxg6f8mdacts24vlkm',
    pool_type: 'standard',
  },
  [keyFromAssets(xARCH, axlUSDC)]: {
    address: 'archway1evz8agrnppzq7gt2nnutkmqgpm86374xds0alc7hru987f9v4hqsejqfaq',
    pool_type: 'standard',
  },
  [keyFromAssets(USDC, IST)]: {
    address: 'archway102gh7tqaeptt88nckg73mfx8j8du64hw4qqm53zwwykcchwar86sza46ge',
    pool_type: 'stable',
  },
  [keyFromAssets(USDC, AXV)]: {
    address: 'archway1fnw37ae9rqs9alu0z9x6jxmd3tnyvnccugks3t28gg5yuv2me4rsxs7gtl',
    pool_type: 'standard',
  },
  [keyFromAssets(xATOM, AXV)]: {
    address: 'archway13kjs3296p4fkzvca2grpueqprfl35er7wefzaa5n8em6u7fd5lcsvue5n3',
    pool_type: 'standard',
  },
  [keyFromAssets(USDC, xJKL)]: {
    address: 'archway1e6l4sh2cqpwvdgp6h99nzv9tn0ga0jvt4qlarxlqsrtfmgpdy84qt8nq32',
    pool_type: 'standard',
  },
  [keyFromAssets(AXV, xJKL)]: {
    address: 'archway1m43xf7zm7p6h8tlagzm82e4tg9zlyah8x3wjha5tf7vsftyysy5qcz4c3l',
    pool_type: 'standard',
  },
};
