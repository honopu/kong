use rand::rngs::StdRng;
use rand::Rng;

use crate::USER_MAP;

const NUM_WORDS: u16 = 184;
const WORDS: [&str; NUM_WORDS as usize] = [
    "able",
    "acting",
    "agile",
    "alert",
    "alive",
    "ample",
    "asking",
    "baking",
    "barring",
    "beautiful",
    "bold",
    "brave",
    "bright",
    "brisk",
    "buoy",
    "bountiful",
    "calm",
    "calling",
    "casting",
    "charm",
    "cheek",
    "cheerful",
    "chic",
    "civil",
    "clear",
    "clever",
    "colorful",
    "cool",
    "cozy",
    "crisp",
    "cute",
    "dandy",
    "dancing",
    "delightful",
    "doubtful",
    "eating",
    "easy",
    "earning",
    "fair",
    "failing",
    "faithful",
    "fancy",
    "fine",
    "fit",
    "forceful",
    "fresh",
    "fretful",
    "fruitful",
    "gaining",
    "glad",
    "gleam",
    "glee",
    "goodness",
    "grand",
    "graceful",
    "grateful",
    "great",
    "hailing",
    "hale",
    "happy",
    "hardy",
    "harmful",
    "heady",
    "heedful",
    "helpful",
    "hefty",
    "hopeful",
    "icing",
    "ireful",
    "joining",
    "joyful",
    "jazzy",
    "jolly",
    "juicy",
    "just",
    "keen",
    "keeping",
    "kind",
    "kicking",
    "kooky",
    "laid",
    "landing",
    "lawful",
    "laughing",
    "lively",
    "loved",
    "loyal",
    "lush",
    "major",
    "meeting",
    "merry",
    "molding",
    "mourning",
    "naming",
    "neat",
    "nifty",
    "noble",
    "noting",
    "okay",
    "open",
    "opting",
    "pairing",
    "peaceful",
    "perky",
    "plentiful",
    "plump",
    "plush",
    "powerful",
    "prime",
    "proud",
    "pure",
    "quitting",
    "quick",
    "quiet",
    "racing",
    "ranging",
    "ready",
    "regal",
    "respectful",
    "right",
    "ripe",
    "robust",
    "rosy",
    "ruthful",
    "safe",
    "sailing",
    "sage",
    "sassy",
    "saucy",
    "saving",
    "scornful",
    "sharp",
    "shiny",
    "skillful",
    "slick",
    "smart",
    "snazzy",
    "snug",
    "sound",
    "sorrowful",
    "spicy",
    "spiteful",
    "stressful",
    "successful",
    "sunny",
    "super",
    "sweet",
    "swift",
    "tactful",
    "taking",
    "taming",
    "tearful",
    "thankful",
    "thoughtful",
    "tidy",
    "tough",
    "tricky",
    "cuddly",
    "trust",
    "uniting",
    "useful",
    "vast",
    "venting",
    "vengeful",
    "waiting",
    "waxing",
    "willful",
    "wise",
    "wining",
    "wonderful",
    "wrathful",
    "yawning",
    "yelling",
    "youthful",
    "zest",
    "zoning",
    "dazzling",
    "buoyant",
    "blissful",
    "jubilation",
    "radiant",
    "serenity",
    "tranquility",
    "nirvana",
];

const NUM_CRYPTO_WORDS: u16 = 314;
const CRYPTO_WORDS: [&str; NUM_CRYPTO_WORDS as usize] = [
    "addr",
    "airdrop",
    "algo",
    "alloc",
    "aml",
    "arch",
    "ask",
    "asic",
    "ath",
    "atl",
    "audit",
    "base",
    "bear",
    "bft",
    "bid",
    "bips",
    "bit",
    "bits",
    "blnd",
    "block",
    "bloc",
    "bridge",
    "brnd",
    "buidl",
    "bull",
    "burn",
    "byte",
    "cap",
    "case",
    "chain",
    "club",
    "code",
    "coin",
    "cold",
    "cons",
    "cohort",
    "cust",
    "cycle",
    "dao",
    "dapp",
    "dash",
    "debt",
    "decent",
    "defi",
    "degen",
    "depo",
    "dex",
    "diff",
    "disk",
    "dlt",
    "dpos",
    "dump",
    "dust",
    "dyor",
    "earn",
    "edit",
    "else",
    "encode",
    "enum",
    "epoch",
    "erc",
    "eth2",
    "ether",
    "ethcc",
    "evm",
    "event",
    "execute",
    "faucet",
    "fiat",
    "file",
    "fin",
    "final",
    "finney",
    "flap",
    "flip",
    "fork",
    "fans",
    "func",
    "fomo",
    "fud",
    "gap",
    "gas",
    "gether",
    "gnt",
    "goto",
    "graph",
    "grid",
    "group",
    "gwei",
    "halving",
    "consensus",
    "hard",
    "hash",
    "heap",
    "hft",
    "hook",
    "hot",
    "hodl",
    "hype",
    "ico",
    "idle",
    "input",
    "inst",
    "ipfs",
    "json",
    "jomo",
    "jump",
    "keys",
    "kill",
    "kyc",
    "kwei",
    "layer1",
    "layer2",
    "lambo",
    "lend",
    "ledger",
    "levr",
    "liqd",
    "link",
    "list",
    "load",
    "lock",
    "long",
    "loop",
    "mainnet",
    "maid",
    "mask",
    "mcap",
    "meme",
    "memp",
    "mempool",
    "merkle",
    "merge",
    "mether",
    "miami",
    "mint",
    "minr",
    "mode",
    "moon",
    "mult",
    "nav",
    "nem",
    "node",
    "nonce",
    "null",
    "offer",
    "omg",
    "opco",
    "open",
    "oracle",
    "orca",
    "order",
    "otc",
    "pack",
    "page",
    "path",
    "peer",
    "pepe",
    "perm",
    "pipe",
    "plng",
    "play",
    "ponz",
    "pool",
    "port",
    "post",
    "proof",
    "pump",
    "push",
    "p2p",
    "query",
    "rack",
    "rare",
    "rank",
    "rate",
    "read",
    "rebase",
    "redeem",
    "rekt",
    "revert",
    "root",
    "roll",
    "rugpull",
    "safu",
    "sats",
    "save",
    "scale",
    "scam",
    "seed",
    "seek",
    "segw",
    "send",
    "shard",
    "share",
    "shift",
    "shill",
    "show",
    "side",
    "sign",
    "silk",
    "size",
    "slot",
    "smart",
    "snst",
    "sodl",
    "solid",
    "sort",
    "span",
    "spend",
    "staking",
    "state",
    "stable",
    "stack",
    "swap",
    "sync",
    "szabo",
    "taps",
    "testnet",
    "tether",
    "time",
    "tint",
    "troll",
    "tool",
    "token",
    "trad",
    "trade",
    "trust",
    "txid",
    "type",
    "unit",
    "unis",
    "usdt",
    "utxo",
    "user",
    "valid",
    "vault",
    "vars",
    "view",
    "vola",
    "vote",
    "wallet",
    "waves",
    "web3",
    "weight",
    "whale",
    "wif",
    "wire",
    "word",
    "wrap",
    "wenmoon",
    "yeld",
    "zero",
    "zil",
    "zone",
    "zksnark",
    "zug",
    "satoshi",
    "nakamoto",
    "canister",
    "motoko",
    "solidity",
    "rust",
    "frens",
    "hodler",
    "doge",
    "shiba",
    "minter",
    "asic",
    "bridge",
    "wormhole",
    "zkrollup",
    "mempool",
    "ordinals",
    "boredapes",
    "punks",
    "chainkey",
    "overflow",
    "barbados",
    "maximalist",
    "multisig",
    "whitehat",
    "whitelist",
    "lfg",
    "mooning",
    "ngmi",
    "wagmi",
    "apeing",
    "btfd",
    "ser",
    "anon",
    "sidechain",
    "arbitrage",
    "base58",
    "web3",
    "goerli",
    "liquidation",
    "loopring",
    "sandwiching",
    "testnet",
    "tokenomics",
    "whitepaper",
    "yieldfarming",
    "protocol",
    "wrapped",
    "deflationary",
    "zksync",
    "sequencer",
    "zkproof",
    "seedphrase",
    "privatekey",
    "altcoin",
    "terrastation",
    "governance",
    "validator",
];

pub fn generate_user_name(rng: &mut StdRng) -> [u16; 3] {
    let mut user_name = [0u16; 3];
    let mut user_name_exists = true;
    while user_name_exists {
        let index1 = rng.gen_range(0..NUM_WORDS);
        let index2 = rng.gen_range(0..NUM_CRYPTO_WORDS);
        let index3 = rng.gen_range(0..NUM_CRYPTO_WORDS);
        if index2 == index3 {
            continue;
        }
        user_name = [index1, index2, index3];
        user_name_exists = USER_MAP.with(|m| {
            m.borrow().iter().map(|(_, v)| v).any(|user| {
                user.user_name[0] == user_name[0]
                    && user.user_name[1] == user_name[1]
                    && user.user_name[2] == user_name[2]
            })
        });
    }
    user_name
}

pub fn to_user_name(indexes: &[u16; 3]) -> String {
    format!(
        "{}-{}-{}",
        WORDS[indexes[0] as usize], CRYPTO_WORDS[indexes[1] as usize], CRYPTO_WORDS[indexes[2] as usize]
    )
}
