// Mock API for FUSION dashboard live data (replace with real fetches)
import { DashboardData } from './types';

export async function fetchDashboardData(): Promise<DashboardData> {
  // Simulate live backend data
  return {
    protocol_stats: [
      { name: 'Venus', profit_threshold: 0.42, scan_interval: 60, weight: 1.0, ai_tuned: true },
      { name: 'Aave', profit_threshold: 0.38, scan_interval: 45, weight: 0.8, ai_tuned: true },
      { name: 'PancakeSwap', profit_threshold: 0.51, scan_interval: 30, weight: 1.2, ai_tuned: false },
    ],
    dex_feed: [
      { timestamp: new Date().toISOString(), dex: 'PancakeSwap', status: 'scanning', message: 'Scanning for arbitrage...' },
      { timestamp: new Date().toISOString(), dex: 'Uniswap', status: 'success', message: 'Found opportunity!' },
      { timestamp: new Date().toISOString(), dex: 'SushiSwap', status: 'error', message: 'API timeout' },
    ],
    liquidation_feed: [
      { timestamp: new Date().toISOString(), account: '0x123...abc', status: 'flagged', details: 'Health factor below 1.0' },
      { timestamp: new Date().toISOString(), account: '0x456...def', status: 'healthy', details: 'No risk' },
      { timestamp: new Date().toISOString(), account: '0x789...fed', status: 'flagged', details: 'Collateral shortfall' },
    ],
    opportunity_metrics: [
      { timestamp: new Date().toISOString(), type: 'arbitrage', dex: 'PancakeSwap', amount: 120.5, status: 'success' },
      { timestamp: new Date().toISOString(), type: 'liquidation', dex: 'Venus', amount: 75.2, status: 'missed' },
    ],
  };
}
