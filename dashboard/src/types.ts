export interface WebSocketEventBase {
  id: string;
  type: string;
  timestamp: string;
}

export interface DexScanEvent extends WebSocketEventBase {
  type: 'dex';
  dex: string;
  message: string;
  status: string;
}

export interface LiquidationEvent extends WebSocketEventBase {
  type: 'liquidation';
  account: string;
  details: string;
  status: string;
}

export interface ParameterUpdateEvent extends WebSocketEventBase {
  type: 'parameter_update';
  parameter_name: string;
  new_value: string;
  source: string;
}

export interface BotStatusEvent extends WebSocketEventBase {
  type: 'bot_status';
  bot_name: string;
  status: string;
  message?: string;
}

export type WebSocketEvent = 
  | DexScanEvent
  | LiquidationEvent
  | ParameterUpdateEvent
  | BotStatusEvent;
