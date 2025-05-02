import React from 'react';
import { DexScanEvent, LiquidationEvent } from '../api/types';

interface LiveFeedsProps {
  dexFeed: DexScanEvent[];
  liquidationFeed: LiquidationEvent[];
}

export const LiveFeeds: React.FC<LiveFeedsProps> = ({ dexFeed, liquidationFeed }) => {
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* DEX Feed */}
        <div className="bg-fusion-dark p-4 rounded-lg">
          <h3 className="text-fusion-accent text-lg font-light tracking-wider mb-2">DEX Scans</h3>
          <div className="space-y-2 h-64 overflow-y-auto">
            {dexFeed.length === 0 && (
              <div className="text-gray-500">No DEX scan events yet...</div>
            )}
            {dexFeed.map((event) => (
              <div key={event.timestamp} className="text-fusion-neon border-b border-fusion-accent/20 pb-1 mb-1 last:border-b-0">
                <div className="font-semibold">{event.dex}</div>
                <div>{event.message}</div>
                <div className="text-xs opacity-75">
                  {new Date(event.timestamp).toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Liquidations Feed */}
        <div className="bg-fusion-dark p-4 rounded-lg">
          <h3 className="text-fusion-accent text-lg font-light tracking-wider mb-2">Liquidations</h3>
          <div className="space-y-2 h-64 overflow-y-auto">
            {liquidationFeed.length === 0 && (
              <div className="text-gray-500">No liquidation events yet...</div>
            )}
            {liquidationFeed.map((event) => (
              <div key={event.timestamp} className="text-fusion-neon border-b border-fusion-accent/20 pb-1 mb-1 last:border-b-0">
                <div className="font-semibold">{event.account}</div>
                <div>{event.details}</div>
                <div className="text-xs opacity-75">
                  {new Date(event.timestamp).toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
