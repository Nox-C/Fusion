// Types for FUSION dashboard live API

export interface ProtocolStat {
  name: string;
  profit_threshold: number;
  scan_interval: number;
  weight: number;
  ai_tuned: boolean;
}

export interface DexScanEvent {
  timestamp: string;
  dex: string;
  status: 'scanning' | 'success' | 'error';
  message: string;
}

export interface LiquidationEvent {
  timestamp: string;
  account: string;
  status: 'flagged' | 'liquidated' | 'healthy';
  details: string;
}

export interface ParameterUpdateEvent {
  type: 'parameter_update';
  parameter_name: string;
  new_value: string;
  timestamp: string;
  source: string;
}

export interface BotStatusEvent {
  type: 'bot_status';
  bot_name: string;
  status: string;
  message?: string;
  timestamp: string;
}

export interface OpportunityMetric {
  timestamp: string;
  type: 'arbitrage' | 'liquidation';
  dex: string;
  amount: number;
  status: 'success' | 'missed';
}

export interface DashboardData {
  protocol_stats: ProtocolStat[];
  dex_feed: DexScanEvent[];
  liquidation_feed: LiquidationEvent[];
  opportunity_metrics: OpportunityMetric[];
}
