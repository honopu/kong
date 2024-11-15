import { getActor, walletStore } from '$lib/services/wallet/walletStore';
import { PoolService } from '../../services/pools/PoolService';
import { formatToNonZeroDecimal, formatTokenAmount } from '$lib/utils/numberFormatUtils';
import { get } from 'svelte/store';
import { ICP_CANISTER_ID } from '$lib/constants/canisterConstants';
import { poolStore } from '$lib/services/pools/poolStore';
import { Principal } from '@dfinity/principal';
import { IcrcService } from '$lib/services/icrc/IcrcService';
import { TokenSerializer } from './TokenSerializer';
import { 
  saveTokenLogo,
  getTokenLogo,
  cleanupExpiredTokenLogos,
  bulkSaveTokenLogos,
  getMultipleTokenLogos,
  fetchTokenLogo,
  DEFAULT_LOGOS
} from './tokenLogo';
import { kongDB } from '../db/db';
import { tokenStore } from './tokenStore';

export class TokenService {
  protected static instance: TokenService;
  private static priceCache = new Map<string, { price: number; timestamp: number }>();
  private static readonly CACHE_DURATION = 5 * 60 * 1000; // 5 minutes
  private static readonly TOKEN_CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

  public static getInstance(): TokenService {
    if (!TokenService.instance) {
      TokenService.instance = new TokenService();
    }
    return TokenService.instance;
  }

  public static async fetchTokens(): Promise<FE.Token[]> {
    try {
      // Try to get cached tokens
      const currentTime = Date.now();
      const cachedTokens = await kongDB.tokens
        .where('timestamp')
        .above(currentTime - this.TOKEN_CACHE_DURATION)
        .toArray();

      if (cachedTokens.length > 0) {
        return cachedTokens;
      }

      // If no valid cache, fetch from network
      const actor = await getActor();
      const result = await actor.tokens(['all']);
      const serialized = TokenSerializer.serializeTokens(result);
      
      if (serialized.Err) throw serialized.Err;
      
      // Cache the results
      await this.cacheTokens(serialized.Ok);
      
      return serialized.Ok;
    } catch (error) {
      console.error('Error fetching tokens:', error);
      throw error;
    }
  }

  private static async cacheTokens(tokens: FE.Token[]): Promise<void> {
    try {
      await kongDB.transaction('rw', kongDB.tokens, async () => {
        // Clear old cache
        await kongDB.tokens.clear();
        
        // Add new tokens with timestamp
        const timestamp = Date.now();
        const cachedTokens: FE.Token[] = tokens.map(token => ({
          ...token,
          timestamp
        }));
        
        await kongDB.tokens.bulkAdd(cachedTokens);
      });
    } catch (error) {
      console.error('Error caching tokens:', error);
    }
  }

  public static async clearTokenCache(): Promise<void> {
    try {
      await kongDB.tokens.clear();
    } catch (error) {
      console.error('Error clearing token cache:', error);
    }
  }

  private static async cleanupExpiredTokens(): Promise<void> {
    try {
      const expirationTime = Date.now() - this.TOKEN_CACHE_DURATION;
      await kongDB.tokens
        .where('timestamp')
        .below(expirationTime)
        .delete();
    } catch (error) {
      console.error('Error cleaning up expired tokens:', error);
    }
  }

  // Optional: Add a method to get a single token from cache
  public static async getToken(canisterId: string): Promise<FE.Token | null> {
    try {
      const currentTime = Date.now();
      const token = await kongDB.tokens
        .where('canisterId')
        .equals(canisterId)
        .and(token => currentTime - token.timestamp < this.TOKEN_CACHE_DURATION)
        .first();
      
      return token || null;
    } catch (error) {
      console.error('Error getting token:', error);
      return null;
    }
  }

  // Optional: Add a method to update a single token in cache
  public static async updateToken(token: FE.Token): Promise<void> {
    try {
      await kongDB.tokens.put({
        ...token,
        timestamp: Date.now()
      });
    } catch (error) {
      console.error('Error updating token:', error);
    }
  }

  public static async enrichTokenWithMetadata(
    tokens: FE.Token[]
  ): Promise<PromiseSettledResult<FE.Token>[]> {
    const poolData = get(poolStore);
    const BATCH_SIZE = 10; // Process 10 tokens at a time

    const processTokenBatch = async (tokenBatch: FE.Token[]) => {
      return Promise.all(tokenBatch.map(async (token) => {
        try {
          const [logo, price, volume24h, fee] = await Promise.allSettled([
            this.getCachedLogo(token),
            this.getCachedPrice(token),
            this.calculate24hVolume(token, poolData.pools),
            token?.fee ? Promise.resolve(token.fee) : this.fetchTokenFee(token),
          ]);

          return {
            ...token,
            fee: fee.status === 'fulfilled' ? fee.value : 0n,
            price: price.status === 'fulfilled' ? price.value : 0,
            total_24h_volume: volume24h.status === 'fulfilled' ? volume24h.value : 0n,
            logo: logo.status === 'fulfilled' ? logo.value : '/tokens/not_verified.webp',
          };
        } catch (error) {
          console.error(`Error enriching token ${token.symbol}:`, error);
          return null;
        }
      }));
    };

    // Process tokens in batches
    const results = [];
    for (let i = 0; i < tokens.length; i += BATCH_SIZE) {
      const batch = tokens.slice(i, i + BATCH_SIZE);
      const batchResults = await processTokenBatch(batch);
      results.push(...batchResults);
    }

    return results.map(r => ({ status: r ? 'fulfilled' : 'rejected', value: r })) as PromiseSettledResult<FE.Token>[];
  }

  private static async getCachedPrice(token: FE.Token): Promise<number> {
    const cached = this.priceCache.get(token.canister_id);
    if (cached && Date.now() - cached.timestamp < this.CACHE_DURATION) {
      return cached.price;
    }

    const price = await this.fetchPrice(token);
    this.priceCache.set(token.canister_id, {
      price,
      timestamp: Date.now()
    });
    return price;
  }

  private static async getCachedLogo(token: FE.Token): Promise<string> {
    // Handle ICP special case first
    if (token.canister_id === ICP_CANISTER_ID) {
      const logo = DEFAULT_LOGOS[ICP_CANISTER_ID];
      await saveTokenLogo(token.canister_id, logo);
      return logo;
    }

    // Try to get from DB cache
    const cachedImage = await getTokenLogo(token.canister_id);
    if (cachedImage) {
      return cachedImage;
    }

    // Fetch from network if not cached
    try {
      const logo = await fetchTokenLogo(token);

      // Validate and cache the logo
      if (logo) {
        await saveTokenLogo(token.canister_id, logo);
        return logo;
      } else {
        await saveTokenLogo(token.canister_id, DEFAULT_LOGOS.DEFAULT);
        return DEFAULT_LOGOS.DEFAULT;
      }
    } catch (error) {
      console.error('Error fetching token logo:', error);
      const defaultLogo = DEFAULT_LOGOS.DEFAULT;
      await saveTokenLogo(token.canister_id, defaultLogo);
      return defaultLogo;
    }
  }

  public static async fetchBalances(
    tokens: FE.Token[],
    principalId: string = null
  ): Promise<Record<string, FE.TokenBalance>> {
    const wallet = get(walletStore);
    if (!wallet.isConnected) return {};

    if (!principalId) {
      return {};
    }

    // Create an array of promises for all tokens
    const balancePromises = tokens.map(async (token) => {
      try {
        let balance: bigint;

        if (token.icrc1 && principalId) {
          balance = await IcrcService.getIcrc1Balance(
            token,
            Principal.fromText(principalId)
          );
        } else {
          // Handle other token types if necessary
          balance = BigInt(0); // Default fallback or handle appropriately
        }

        const actualBalance = formatTokenAmount(balance.toString(), token.decimals);
        const price = await this.fetchPrice(token);
        const usdValue = parseFloat(actualBalance) * price;

        return {
          canister_id: token.canister_id,
          balance: {
            in_tokens: balance || BigInt(0),
            in_usd: formatToNonZeroDecimal(usdValue),
          }
        };
      } catch (err) {
        console.error(`Error fetching balance for ${token.canister_id}:`, err);
        console.log("principalId", principalId.toString());
        console.log("token", token);
        // Optionally provide more details from 'err'
        return {
          canister_id: token.canister_id,
          balance: {
            in_tokens: BigInt(0),
            in_usd: formatToNonZeroDecimal(0),
          }
        };
      }
    });

    // Wait for all balance promises to resolve
    const resolvedBalances = await Promise.allSettled(balancePromises);

    // Convert the array of results into a record
    const balances: Record<string, FE.TokenBalance> = {};
    resolvedBalances.forEach((result) => {
      if (result.status === 'fulfilled') {
        const { canister_id, balance } = result.value;
        balances[canister_id] = balance;
      }
    });

    return balances;
  }

  public static async fetchBalance(token: FE.Token, principalId?: string, forceRefresh = false): Promise<FE.TokenBalance> {
    // Add cache check
    if (!forceRefresh) {
        const cachedBalance = get(tokenStore).balances[token.canister_id];
        if (cachedBalance) {
            return cachedBalance;
        }
    }

    if (!token?.canister_id) {
        return {
            in_tokens: BigInt(0),
            in_usd: formatToNonZeroDecimal(0),
        };
    }

    const balance = await IcrcService.getIcrc1Balance(
        token,
        Principal.fromText(principalId)
    );
  
    const actualBalance = formatTokenAmount(balance.toString(), token.decimals);
    const price = await this.fetchPrice(token);
    const usdValue = parseFloat(actualBalance) * price;

    return {
        in_tokens: balance || BigInt(0),
        in_usd: formatToNonZeroDecimal(usdValue),
    };
  }

  public static async fetchPrices(tokens: FE.Token[]): Promise<Record<string, number>> {
    const poolData = await PoolService.fetchPoolsData();

    // Create an array of promises for all tokens
    const pricePromises = tokens.map(async (token) => {
      const usdtPool = poolData.pools.find(pool => 
        (pool.address_0 === token.canister_id && pool.symbol_1 === "ckUSDT") ||
        (pool.address_1 === token.canister_id && pool.symbol_0 === "ckUSDT")
      );

      if (usdtPool) {
        return { canister_id: token.canister_id, price: usdtPool.price };
      } else {
        const icpPool = poolData.pools.find(pool => 
          (pool.address_0 === token.canister_id && pool.symbol_1 === "ICP")
        );

        if (icpPool) {
          const icpUsdtPrice = await this.getUsdtPriceForToken("ICP", poolData.pools);
          return { canister_id: token.canister_id, price: icpUsdtPrice * icpPool.price };
        } else {
          return { canister_id: token.canister_id, price: 0 };
        }
      }
    });

    // Wait for all price promises to resolve
    const resolvedPrices = await Promise.allSettled(pricePromises);

    // Convert the array of results into a record
    const prices: Record<string, number> = {};
    // insert into dexie
    resolvedPrices.forEach((result) => {
      if (result.status === 'fulfilled') {
        const { canister_id, price } = result.value;
        prices[canister_id] = price;
      }
    });
    prices[process.env.CANISTER_ID_CKUSDT_LEDGER] = 1;

    await kongDB.tokens.bulkPut(tokens, ['canister_id'], );
    return prices;
  }

  public static async fetchPrice(token: FE.Token): Promise<number> {
    const poolData = get(poolStore);
    
    const relevantPools = poolData.pools.filter(pool => 
        pool.address_0 === token.canister_id || 
        pool.address_1 === token.canister_id
    );

    if (relevantPools.length === 0) return 0;

    let totalWeight = 0n;
    let weightedPrice = 0;

    for (const pool of relevantPools) {
        let price: number;
        let weight: bigint;

        if (pool.address_0 === token.canister_id) {
            if (pool.symbol_1 === "ICP") {
              const icpPrice = await this.getUsdtPriceForToken("ICP", poolData.pools);
              const usdtPrice = pool.price * icpPrice;
              price = usdtPrice;
            } else {
                price = pool.price * (await this.getUsdtPriceForToken(pool.symbol_1, poolData.pools));
            }
            weight = pool.balance_0;
        } else {
            if (pool.symbol_0 === "ckUSDT") {
                price = 1 / pool.price;
            } else {
                price = (1 / pool.price) * (await this.getUsdtPriceForToken(pool.symbol_0, poolData.pools));
            }
            weight = pool.balance_1;
        }

        if (price > 0 && weight > 0n) {
            weightedPrice += Number(weight) * price;
            totalWeight += weight;
        }
    }

    return totalWeight > 0n ? weightedPrice / Number(totalWeight) : 0;
  }

  private static async getUsdtPriceForToken(symbol: string, pools: BE.Pool[]): Promise<number> {
    const usdtPool = pools.find(
      (p) =>
        (p.symbol_0 === symbol && p.symbol_1 === 'ckUSDT') ||
        (p.symbol_1 === symbol && p.symbol_0 === 'ckUSDT')
    );

    if (usdtPool) {
      const price =
        usdtPool.symbol_1 === 'ckUSDT'
          ? usdtPool.price
          : 1 / usdtPool.price;
      return price;
    }

    const icpPrice = await this.getUsdtPriceViaICP(symbol, pools);
    if (icpPrice > 0) {
      return icpPrice;
    }

    console.warn(`Unable to determine USDT price for token: ${symbol}`);
    return 0;
  }

  private static async getUsdtPriceViaICP(symbol: string, pools: BE.Pool[]): Promise<number> {
    const tokenIcpPool = pools.find(
      (p) =>
        (p.symbol_0 === symbol && p.symbol_1 === 'ICP') ||
        (p.symbol_1 === symbol && p.symbol_0 === 'ICP')
    );

    const icpUsdtPool = pools.find(
      (p) =>
        (p.symbol_0 === 'ICP' && p.symbol_1 === 'ckUSDT') ||
        (p.symbol_1 === 'ICP' && p.symbol_0 === 'ckUSDT')
    );

    if (tokenIcpPool && icpUsdtPool) {
      const tokenPriceInIcp =
        tokenIcpPool.symbol_1 === 'ICP'
          ? tokenIcpPool.price
          : 1 / tokenIcpPool.price;

      const icpPriceInUsdt =
        icpUsdtPool.symbol_1 === 'ckUSDT'
          ? icpUsdtPool.price
          : 1 / icpUsdtPool.price;

      const combinedPrice = tokenPriceInIcp * icpPriceInUsdt;
      return combinedPrice;
    }

    console.warn(`No ICP pools found for token: ${symbol}`);
    return 0;
  }

  public static async getIcrc1TokenMetadata(canisterId: string): Promise<any> {
    try {
      const actor = await getActor(canisterId, 'icrc1');
      return await actor.icrc1_metadata();
    } catch (error) {
      console.error('Error getting icrc1 token metadata:', error);
      throw error;
    }
  }

  public static async fetchUserTransactions(principalId: string, tokenId = ""): Promise<any> {
    const actor = await getActor();
    return await actor.txs([true]);
  }

  public static async claimFaucetTokens(): Promise<any> {
    try {
      const kongFaucetId = process.env.CANISTER_ID_KONG_FAUCET;
      const actor = await getActor(kongFaucetId, 'kong_faucet');
      return await actor.claim();
    } catch (error) {
      console.error('Error claiming faucet tokens:', error);
    }
  }

  private static async calculate24hVolume(token: FE.Token, pools: BE.Pool[]): Promise<bigint> {
    let total24hVolume = 0n;

    pools.forEach(pool => {
      if (pool.address_0 === token.canister_id || pool.address_1 === token.canister_id) {
        if (pool.rolling_24h_volume) {
          total24hVolume += pool.rolling_24h_volume;
        }
      }
    });

    return total24hVolume;
  }

  private static async fetchTokenFee(token: FE.Token): Promise<bigint> {
    try {
      if (token.canister_id === ICP_CANISTER_ID) {
        // For ICP, use the standard transaction fee
        // ICP's fee is typically 10,000 e8s (0.0001 ICP)
        return BigInt(10000);
      } else {
        const actor = await getActor(token.canister_id, 'icrc1');
        const fee = await actor.icrc1_fee();
        return fee;
      }
    } catch (error) {
      console.error(`Error fetching fee for ${token.symbol}:`, error);
      // Provide a default fee if necessary
      return BigInt(10000); // Adjust default fee as appropriate
    }
  }
} 
