// src/lib/services/swap/SwapService.ts
import {
  getTokenDecimals,
  liveTokens,
  loadBalances,
} from "$lib/services/tokens/tokenStore";
import { toastStore } from "$lib/stores/toastStore";
import { get } from "svelte/store";
import { Principal } from "@dfinity/principal";
import BigNumber from "bignumber.js";
import { IcrcService } from "$lib/services/icrc/IcrcService";
import { swapStatusStore } from "./swapStore";
import { auth, canisterIDLs } from "$lib/services/auth";
import { KONG_BACKEND_CANISTER_ID } from "$lib/constants/canisterConstants";
import { requireWalletConnection } from "$lib/services/auth";

interface SwapExecuteParams {
  swapId: string;
  payToken: FE.Token;
  payAmount: string;
  receiveToken: FE.Token;
  receiveAmount: string;
  userMaxSlippage: number;
  backendPrincipal: Principal;
  lpFees: any[];
}

interface SwapStatus {
  status: string;
  pay_amount: bigint;
  pay_symbol: string;
  receive_amount: bigint;
  receive_symbol: string;
}

interface SwapResponse {
  Swap: SwapStatus;
}

interface RequestStatus {
  reply: SwapResponse;
  statuses: string[];
}

interface RequestResponse {
  Ok: RequestStatus[];
}

interface TokenInfo {
  canister_id: string;
  fee?: bigint;
  fee_fixed: bigint;
  icrc1?: boolean;
  icrc2?: boolean;
}

// Base BigNumber configuration for internal calculations
// Set this high enough to handle intermediate calculations without loss of precision
BigNumber.config({
  DECIMAL_PLACES: 36, // High enough for internal calculations
  ROUNDING_MODE: BigNumber.ROUND_DOWN,
  EXPONENTIAL_AT: [-50, 50],
});

export class SwapService {
  private static INITIAL_POLLING_INTERVAL = 500; // 500ms initially
  private static FAST_POLLING_INTERVAL = 100; // 100ms after 5 seconds
  private static FAST_POLLING_DELAY = 5000; // 5 seconds before switching to fast polling
  private static pollingInterval: NodeJS.Timeout | null = null;
  private static startTime: number;
  private static readonly POLLING_INTERVAL = 300; // .3 second
  private static readonly MAX_ATTEMPTS = 200; // 30 seconds

  private static isValidNumber(value: string | number): boolean {
    if (typeof value === "number") {
      return !isNaN(value) && isFinite(value);
    }
    if (typeof value === "string") {
      const num = Number(value);
      return !isNaN(num) && isFinite(num);
    }
    return false;
  }

  public static toBigInt(
    value: string | number | BigNumber,
    decimals?: number,
  ): bigint {
    try {
      // If decimals provided, handle scaling
      if (decimals !== undefined) {
        const multiplier = new BigNumber(10).pow(decimals);

        // If it's a BigNumber instance
        if (value instanceof BigNumber) {
          if (value.isNaN() || !value.isFinite()) {
            console.warn("Invalid BigNumber value:", value);
            return BigInt(0);
          }
          return BigInt(
            value
              .times(multiplier)
              .integerValue(BigNumber.ROUND_DOWN)
              .toString(),
          );
        }

        // If it's a string or number
        if (!this.isValidNumber(value)) {
          console.warn("Invalid numeric value:", value);
          return BigInt(0);
        }

        const bn = new BigNumber(value);
        if (bn.isNaN() || !bn.isFinite()) {
          console.warn("Invalid conversion to BigNumber:", value);
          return BigInt(0);
        }

        return BigInt(
          bn.times(multiplier).integerValue(BigNumber.ROUND_DOWN).toString(),
        );
      }

      // Original logic for when no decimals provided
      if (value instanceof BigNumber) {
        return BigInt(value.integerValue(BigNumber.ROUND_DOWN).toString());
      }

      if (!this.isValidNumber(value)) {
        return BigInt(0);
      }

      const bn = new BigNumber(value);
      return BigInt(bn.integerValue(BigNumber.ROUND_DOWN).toString());
    } catch (error) {
      console.error("Error converting to BigInt:", error);
      return BigInt(0);
    }
  }

  public static fromBigInt(amount: bigint, decimals: number): string {
    try {
      const result = new BigNumber(amount.toString())
        .div(new BigNumber(10).pow(decimals))
        .toString();
      return isNaN(Number(result)) ? "0" : result;
    } catch {
      return "0";
    }
  }

  /**
   * Gets swap quote from backend
   */
  public static async swap_amounts(
    payToken: FE.Token,
    payAmount: bigint,
    receiveToken: FE.Token,
  ): Promise<BE.SwapQuoteResponse> {
    try {
      if (!payToken?.symbol || !receiveToken?.symbol) {
        throw new Error("Invalid tokens provided for swap quote");
      }
      const actor = await auth.getActor(
        KONG_BACKEND_CANISTER_ID,
        canisterIDLs.kong_backend,
        { anon: true },
      );
      return await actor.swap_amounts(
        payToken.symbol,
        payAmount,
        receiveToken.symbol,
      );
    } catch (error) {
      console.error("Error getting swap amounts:", error);
      throw error;
    }
  }

  /**
   * Gets quote details including price, fees, etc.
   */
  public static async getQuoteDetails(params: {
    payToken: FE.Token;
    payAmount: bigint;
    receiveToken: FE.Token;
  }): Promise<{
    receiveAmount: string;
    price: string;
    usdValue: string;
    lpFee: String;
    gasFee: String;
    tokenFee?: String;
    slippage: number;
  }> {
    const quote = await SwapService.swap_amounts(
      params.payToken,
      params.payAmount,
      params.receiveToken,
    );

    if (!("Ok" in quote)) {
      throw new Error(quote.Err);
    }

    const tokens = get(liveTokens);
    const receiveToken = tokens.find(
      (t) => t.address === params.receiveToken.address,
    );
    const payToken = tokens.find((t) => t.address === params.payToken.address);
    if (!receiveToken) throw new Error("Receive token not found");

    const receiveAmount = SwapService.fromBigInt(
      quote.Ok.receive_amount,
      receiveToken.decimals,
    );

    let lpFee = "0";
    let gasFee = "0";
    let tokenFee = "0";

    if (quote.Ok.txs.length > 0) {
      const tx = quote.Ok.txs[0];
      lpFee = SwapService.fromBigInt(tx.lp_fee, receiveToken.decimals);
      gasFee = SwapService.fromBigInt(tx.gas_fee, receiveToken.decimals);
      tokenFee = payToken.fee_fixed.toString();
    }

    return {
      receiveAmount,
      price: quote.Ok.price.toString(),
      usdValue: new BigNumber(receiveAmount).times(quote.Ok.price).toFormat(2),
      lpFee,
      gasFee,
      tokenFee,
      slippage: quote.Ok.slippage,
    };
  }

  /**
   * Executes swap asynchronously
   */
  public static async swap_async(params: {
    pay_token: string;
    pay_amount: bigint;
    receive_token: string;
    receive_amount: bigint[];
    max_slippage: number[];
    receive_address?: string[];
    referred_by?: string[];
    pay_tx_id?: { BlockIndex: number }[];
  }): Promise<BE.SwapAsyncResponse> {
    try {
      const actor = await auth.pnp.getActor(
        KONG_BACKEND_CANISTER_ID,
        canisterIDLs.kong_backend,
        {
          anon: false,
          requiresSigning: auth.pnp.activeWallet.id === "plug",
        },
      );
      const result = await actor.swap_async(params);
      return result;
    } catch (error) {
      console.error("Error in swap_async:", error);
      throw error;
    }
  }

  /**
   * Gets request status
   */
  public static async requests(requestIds: bigint[]): Promise<RequestResponse> {
    try {
      const actor = await auth.pnp.getActor(
        KONG_BACKEND_CANISTER_ID,
        canisterIDLs.kong_backend,
        { anon: true },
      );
      const result = await actor.requests(requestIds);
      return result;
    } catch (error) {
      console.error("Error getting request status:", error);
      throw error;
    }
  }

  /**
   * Executes complete swap flow
   */
  public static async executeSwap(
    params: SwapExecuteParams,
  ): Promise<bigint | false> {
    const swapId = params.swapId;
    try {
      requireWalletConnection();
      const tokens = get(liveTokens);
      const payToken = tokens.find(
        (t) => t.canister_id === params.payToken.canister_id,
      );

      if (!payToken) {
        console.error("Pay token not found:", params.payToken);
        throw new Error(`Pay token ${params.payToken.symbol} not found`);
      }

      const payAmount = SwapService.toBigInt(
        params.payAmount,
        payToken.decimals,
      );

      const receiveToken = tokens.find(
        (t) => t.canister_id === params.receiveToken.canister_id,
      );

      if (!receiveToken) {
        console.error("Receive token not found:", params.receiveToken);
        throw new Error(
          `Receive token ${params.receiveToken.symbol} not found`,
        );
      }

      const receiveAmount = SwapService.toBigInt(
        params.receiveAmount,
        receiveToken.decimals,
      );

      let txId: bigint | false;
      let approvalId: bigint | false;
      const toastId = toastStore.info(
        `Swapping ${params.payAmount} ${params.payToken.symbol} to ${params.receiveAmount} ${params.receiveToken.symbol}...`,
      );
      if (payToken.icrc2) {
        const requiredAllowance = payAmount;
        approvalId = await IcrcService.checkAndRequestIcrc2Allowances(
          payToken,
          requiredAllowance,
        );
      } else if (payToken.icrc1) {
        const result = await IcrcService.transfer(
          payToken,
          params.backendPrincipal,
          payAmount,
          { fee: BigInt(payToken.fee_fixed) },
        );

        if (result?.Ok) {
          txId = result.Ok;
        } else {
          txId = false;
        }
      } else {
        throw new Error(
          `Token ${payToken.symbol} does not support ICRC1 or ICRC2`,
        );
      }

      if (txId === false || approvalId === false) {
        swapStatusStore.updateSwap(swapId, {
          status: "Failed",
          isProcessing: false,
          error: "Transaction failed during transfer/approval",
        });
        toastStore.error("Transaction failed during transfer/approval");
        return false;
      }

      const swapParams = {
        pay_token: params.payToken.symbol,
        pay_amount: BigInt(payAmount),
        receive_token: params.receiveToken.symbol,
        receive_amount: [],
        max_slippage: [params.userMaxSlippage],
        receive_address: [],
        referred_by: [],
        pay_tx_id: txId ? [{ BlockIndex: Number(txId) }] : [],
      };

      const result = await SwapService.swap_async(swapParams);

      if (result.Ok) {
        this.monitorTransaction(result?.Ok, swapId);
      } else {
        console.error("Swap error:", result.Err);
        return false;
      }
      return result.Ok;
    } catch (error) {
      swapStatusStore.updateSwap(swapId, {
        status: "Failed",
        isProcessing: false,
        error: error instanceof Error ? error.message : "Swap failed",
      });
      console.error("Swap execution failed:", error);
      toastStore.error(error instanceof Error ? error.message : "Swap failed");
      return false;
    }
  }

  public static async monitorTransaction(requestId: bigint, swapId: string) {
    this.stopPolling();
    this.startTime = Date.now();
    let attempts = 0;
    let lastStatus = ""; // Track the last status
    let swapStatus = swapStatusStore.getSwap(swapId);
    toastStore.info(
      `Confirming swap of ${swapStatus?.payToken.symbol} to ${swapStatus?.receiveToken.symbol}...`,
      { duration: 10000 },
    );

    const poll = async () => {
      if (attempts >= this.MAX_ATTEMPTS) {
        this.stopPolling();
        swapStatusStore.updateSwap(swapId, {
          status: "Timeout",
          isProcessing: false,
          error: "Swap timed out",
        });
        toastStore.error("Swap timed out");
        return;
      }

      try {
        const status: RequestResponse = await this.requests([requestId]);

        if (status.Ok?.[0]?.reply) {
          const res: RequestStatus = status.Ok[0];

          // Only show toast for new status updates
          if (res.statuses && res.statuses.length > 0) {
            const latestStatus = res.statuses[res.statuses.length - 1];
            if (latestStatus !== lastStatus) {
              lastStatus = latestStatus;
              if (latestStatus.includes("Success")) {
                toastStore.success(`Swap completed successfully`);
              } else if (res.statuses.length == 1) {
                toastStore.info(`${latestStatus}`);
              } else if (latestStatus.includes("Failed")) {
                toastStore.error(`${latestStatus}`);
              }
            }
          }

          if (res.statuses.find((s) => s.includes("Failed"))) {
            this.stopPolling();
            swapStatusStore.updateSwap(swapId, {
              status: "Error",
              isProcessing: false,
              error: res.statuses.find((s) => s.includes("Failed")),
            });
            toastStore.error(res.statuses.find((s) => s.includes("Failed")));
            return;
          }

          if ("Swap" in res.reply) {
            const swapStatus: SwapStatus = res.reply.Swap;
            swapStatusStore.updateSwap(swapId, {
              status: swapStatus.status,
              isProcessing: true,
              error: null,
            });

            if (swapStatus.status === "Success") {
              this.stopPolling();
              const token0 = get(liveTokens).find(
                (t) => t.symbol === swapStatus.pay_symbol,
              );
              const token1 = get(liveTokens).find(
                (t) => t.symbol === swapStatus.receive_symbol,
              );

              const formattedPayAmount = SwapService.fromBigInt(
                swapStatus.pay_amount,
                token0?.decimals || 0,
              );
              const formattedReceiveAmount = SwapService.fromBigInt(
                swapStatus.receive_amount,
                token1?.decimals || 0,
              );

              swapStatusStore.updateSwap(swapId, {
                status: "Success",
                isProcessing: false,
                shouldRefreshQuote: true,
                lastQuote: null,
                details: {
                  payAmount: formattedPayAmount,
                  payToken: token0,
                  receiveAmount: formattedReceiveAmount,
                  receiveToken: token1,
                },
              });

              // Load updated balances immediately and after delays
              const tokens = get(liveTokens);
              const payToken = tokens.find(
                (t) => t.symbol === swapStatus.pay_symbol,
              );
              const receiveToken = tokens.find(
                (t) => t.symbol === swapStatus.receive_symbol,
              );
              const walletId = auth?.pnp?.account?.owner?.toString();

              if (!payToken || !receiveToken || !walletId) {
                console.error(
                  "Missing token or wallet info for balance update",
                );
                return;
              }

              const updateBalances = async () => {
                try {
                  await loadBalances(walletId.toString(), {
                    tokens: [payToken, receiveToken],
                    forceRefresh: true,
                  });
                } catch (error) {
                  console.error("Error updating balances:", error);
                }
              };

              // Update immediately
              await updateBalances();

              // Schedule updates with increasing delays
              const delays = [1000, 2000, 3000, 3000, 3000, 5000];
              delays.forEach((delay) => {
                setTimeout(async () => {
                  await updateBalances();
                }, delay);
              });

              return;
            } else if (swapStatus.status === "Failed") {
              this.stopPolling();
              swapStatusStore.updateSwap(swapId, {
                status: "Failed",
                isProcessing: false,
                error: "Swap failed",
              });
              toastStore.error("Swap failed");
              return;
            }
          }
        }

        attempts++;

        // Calculate next polling interval
        const elapsedTime = Date.now() - this.startTime;
        const nextInterval =
          elapsedTime >= this.FAST_POLLING_DELAY
            ? this.FAST_POLLING_INTERVAL
            : this.INITIAL_POLLING_INTERVAL;

        // Schedule next poll
        this.pollingInterval = setTimeout(poll, nextInterval);
      } catch (error) {
        console.error("Error monitoring swap:", error);
        this.stopPolling();
        swapStatusStore.updateSwap(swapId, {
          status: "Error",
          isProcessing: false,
          error: "Failed to monitor swap status",
        });
        toastStore.error("Failed to monitor swap status");
        return;
      }
    };

    // Start polling
    poll();
  }

  private static stopPolling() {
    if (this.pollingInterval) {
      clearTimeout(this.pollingInterval);
      this.pollingInterval = null;
    }
  }

  // Clean up method to be called when component unmounts
  public static cleanup() {
    this.stopPolling();
  }

  /**
   * Fetches the swap quote based on the provided amount and tokens.
   */
  public static async getSwapQuote(
    payToken: FE.Token,
    receiveToken: FE.Token,
    payAmount: string,
  ): Promise<{ receiveAmount: string; slippage: number }> {
    try {
      // Validate input amount
      if (!payAmount || isNaN(Number(payAmount))) {
        console.warn("Invalid pay amount:", payAmount);
        return {
          receiveAmount: "0",
          slippage: 0,
        };
      }

      // Convert amount to BigInt with proper decimal handling
      const payAmountBN = new BigNumber(payAmount);
      const payAmountInTokens = this.toBigInt(payAmountBN, payToken.decimals);

      const quote = await this.swap_amounts(
        payToken,
        payAmountInTokens,
        receiveToken,
      );

      if ("Ok" in quote) {
        // Get decimals synchronously from the token object
        const receiveDecimals = receiveToken.decimals;
        const receivedAmount = this.fromBigInt(
          quote.Ok.receive_amount,
          receiveDecimals,
        );

        return {
          receiveAmount: receivedAmount,
          slippage: quote.Ok.slippage,
        };
      } else if ("Err" in quote) {
        throw new Error(quote.Err);
      }

      throw new Error("Invalid quote response");
    } catch (err) {
      console.error("Error fetching swap quote:", err);
      throw err;
    }
  }

  /**
   * Calculates the maximum transferable amount of a token, considering fees.
   *
   * @param tokenInfo - Information about the token, including fees and canister ID.
   * @param formattedBalance - The user's available balance of the token as a string.
   * @param decimals - Number of decimal places the token supports.
   * @param isIcrc1 - Boolean indicating if the token follows the ICRC1 standard.
   * @returns A BigNumber representing the maximum transferable amount.
   */
  public static calculateMaxAmount(
    tokenInfo: TokenInfo,
    formattedBalance: string,
    decimals: number = 8,
    isIcrc1: boolean = false,
  ): BigNumber {
    const SCALE_FACTOR = new BigNumber(10).pow(decimals);
    const balance = new BigNumber(formattedBalance);

    // Calculate base fee. If fee is undefined, default to 0.
    const baseFee = tokenInfo.fee_fixed
      ? new BigNumber(tokenInfo.fee_fixed.toString()).dividedBy(SCALE_FACTOR)
      : new BigNumber(0);

    // Calculate gas fee based on token standard
    const gasFee = isIcrc1 ? baseFee : baseFee.multipliedBy(2);

    // Ensure that the max amount is not negative
    const maxAmount = balance.minus(gasFee);
    return BigNumber.maximum(maxAmount, new BigNumber(0));
  }
}
