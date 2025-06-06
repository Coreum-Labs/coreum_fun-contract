/**
* This file was automatically generated by @cosmwasm/ts-codegen@1.12.1.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Uint128, InstantiateMsg, ExecuteMsg, DrawState, QueryMsg, BalanceResponse, AccumulatedRewardsResponse, AccumulatedRewardsAtUndelegationResponse, BonusRewardsResponse, ClaimsResponse, ClaimInfo, CurrentStateResponse, DelegatedAmountResponse, Coin, DraftTvlResponse, TicketsSoldResponse, ParticipantsResponse, ParticipantInfo, TicketHoldersResponse, TotalBurnedResponse, UserTicketsResponse, UserWinChanceResponse, WinnerResponse } from "./CoreumFun.types";
export interface CoreumFunReadOnlyInterface {
  contractAddress: string;
  balance: ({
    account
  }: {
    account: string;
  }) => Promise<BalanceResponse>;
  getParticipants: () => Promise<ParticipantsResponse>;
  getWinner: () => Promise<WinnerResponse>;
  getCurrentState: () => Promise<CurrentStateResponse>;
  getNumberOfTicketsSold: () => Promise<TicketsSoldResponse>;
  getBonusRewards: () => Promise<BonusRewardsResponse>;
  getAccumulatedRewards: () => Promise<AccumulatedRewardsResponse>;
  getDraftTvl: () => Promise<DraftTvlResponse>;
  getTicketHolders: () => Promise<TicketHoldersResponse>;
  getUserNumberOfTickets: ({
    address
  }: {
    address: string;
  }) => Promise<UserTicketsResponse>;
  getUserWinChance: ({
    address
  }: {
    address: string;
  }) => Promise<UserWinChanceResponse>;
  getTotalTicketsBurned: () => Promise<TotalBurnedResponse>;
  getClaims: ({
    address
  }: {
    address?: string;
  }) => Promise<ClaimsResponse>;
  getDelegatedAmount: () => Promise<DelegatedAmountResponse>;
  getAccumulatedRewardsAtUndelegation: () => Promise<AccumulatedRewardsAtUndelegationResponse>;
}
export class CoreumFunQueryClient implements CoreumFunReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;
  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.balance = this.balance.bind(this);
    this.getParticipants = this.getParticipants.bind(this);
    this.getWinner = this.getWinner.bind(this);
    this.getCurrentState = this.getCurrentState.bind(this);
    this.getNumberOfTicketsSold = this.getNumberOfTicketsSold.bind(this);
    this.getBonusRewards = this.getBonusRewards.bind(this);
    this.getAccumulatedRewards = this.getAccumulatedRewards.bind(this);
    this.getDraftTvl = this.getDraftTvl.bind(this);
    this.getTicketHolders = this.getTicketHolders.bind(this);
    this.getUserNumberOfTickets = this.getUserNumberOfTickets.bind(this);
    this.getUserWinChance = this.getUserWinChance.bind(this);
    this.getTotalTicketsBurned = this.getTotalTicketsBurned.bind(this);
    this.getClaims = this.getClaims.bind(this);
    this.getDelegatedAmount = this.getDelegatedAmount.bind(this);
    this.getAccumulatedRewardsAtUndelegation = this.getAccumulatedRewardsAtUndelegation.bind(this);
  }
  balance = async ({
    account
  }: {
    account: string;
  }): Promise<BalanceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      balance: {
        account
      }
    });
  };
  getParticipants = async (): Promise<ParticipantsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_participants: {}
    });
  };
  getWinner = async (): Promise<WinnerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_winner: {}
    });
  };
  getCurrentState = async (): Promise<CurrentStateResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_current_state: {}
    });
  };
  getNumberOfTicketsSold = async (): Promise<TicketsSoldResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_number_of_tickets_sold: {}
    });
  };
  getBonusRewards = async (): Promise<BonusRewardsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_bonus_rewards: {}
    });
  };
  getAccumulatedRewards = async (): Promise<AccumulatedRewardsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_accumulated_rewards: {}
    });
  };
  getDraftTvl = async (): Promise<DraftTvlResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_draft_tvl: {}
    });
  };
  getTicketHolders = async (): Promise<TicketHoldersResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_ticket_holders: {}
    });
  };
  getUserNumberOfTickets = async ({
    address
  }: {
    address: string;
  }): Promise<UserTicketsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_user_number_of_tickets: {
        address
      }
    });
  };
  getUserWinChance = async ({
    address
  }: {
    address: string;
  }): Promise<UserWinChanceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_user_win_chance: {
        address
      }
    });
  };
  getTotalTicketsBurned = async (): Promise<TotalBurnedResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_total_tickets_burned: {}
    });
  };
  getClaims = async ({
    address
  }: {
    address?: string;
  }): Promise<ClaimsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_claims: {
        address
      }
    });
  };
  getDelegatedAmount = async (): Promise<DelegatedAmountResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_delegated_amount: {}
    });
  };
  getAccumulatedRewardsAtUndelegation = async (): Promise<AccumulatedRewardsAtUndelegationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_accumulated_rewards_at_undelegation: {}
    });
  };
}
export interface CoreumFunInterface extends CoreumFunReadOnlyInterface {
  contractAddress: string;
  sender: string;
  buyTicket: ({
    numberOfTickets
  }: {
    numberOfTickets: Uint128;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  selectWinnerAndUndelegate: ({
    winnerAddress
  }: {
    winnerAddress: string;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  sendFunds: ({
    amount,
    recipient
  }: {
    amount: Uint128;
    recipient: string;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  burnTickets: ({
    numberOfTickets
  }: {
    numberOfTickets: Uint128;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  addBonusRewardToThePool: ({
    amount
  }: {
    amount: Uint128;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  updateDrawState: ({
    newState
  }: {
    newState: DrawState;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  setUndelegationTimestamp: ({
    timestamp
  }: {
    timestamp: number;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  sendFundsToWinner: (fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
}
export class CoreumFunClient extends CoreumFunQueryClient implements CoreumFunInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;
  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.buyTicket = this.buyTicket.bind(this);
    this.selectWinnerAndUndelegate = this.selectWinnerAndUndelegate.bind(this);
    this.sendFunds = this.sendFunds.bind(this);
    this.burnTickets = this.burnTickets.bind(this);
    this.addBonusRewardToThePool = this.addBonusRewardToThePool.bind(this);
    this.updateDrawState = this.updateDrawState.bind(this);
    this.setUndelegationTimestamp = this.setUndelegationTimestamp.bind(this);
    this.sendFundsToWinner = this.sendFundsToWinner.bind(this);
  }
  buyTicket = async ({
    numberOfTickets
  }: {
    numberOfTickets: Uint128;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      buy_ticket: {
        number_of_tickets: numberOfTickets
      }
    }, fee_, memo_, funds_);
  };
  selectWinnerAndUndelegate = async ({
    winnerAddress
  }: {
    winnerAddress: string;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      select_winner_and_undelegate: {
        winner_address: winnerAddress
      }
    }, fee_, memo_, funds_);
  };
  sendFunds = async ({
    amount,
    recipient
  }: {
    amount: Uint128;
    recipient: string;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      send_funds: {
        amount,
        recipient
      }
    }, fee_, memo_, funds_);
  };
  burnTickets = async ({
    numberOfTickets
  }: {
    numberOfTickets: Uint128;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      burn_tickets: {
        number_of_tickets: numberOfTickets
      }
    }, fee_, memo_, funds_);
  };
  addBonusRewardToThePool = async ({
    amount
  }: {
    amount: Uint128;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      add_bonus_reward_to_the_pool: {
        amount
      }
    }, fee_, memo_, funds_);
  };
  updateDrawState = async ({
    newState
  }: {
    newState: DrawState;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_draw_state: {
        new_state: newState
      }
    }, fee_, memo_, funds_);
  };
  setUndelegationTimestamp = async ({
    timestamp
  }: {
    timestamp: number;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_undelegation_timestamp: {
        timestamp
      }
    }, fee_, memo_, funds_);
  };
  sendFundsToWinner = async (fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      send_funds_to_winner: {}
    }, fee_, memo_, funds_);
  };
}