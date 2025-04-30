import React, { useEffect, useState } from 'react';
import { DexScanEvent, LiquidationEvent } from '../api/types';
import { fetchDashboardData } from '../api/mock';

export default function LiveFeeds() {
  const [dexFeed, setDexFeed] = useState<DexScanEvent[]>([]);
  const [liqFeed, setLiqFeed] = useState<LiquidationEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchDashboardData()
      .then(data => {
        setDexFeed(data.dex_feed);
        setLiqFeed(data.liquidation_feed);
        setLoading(false);
      })
      .catch(err => {
        setError('Failed to load live feeds');
        setLoading(false);
      });
  }, []);

  if (loading) return <div className="text-fusion-neon">Loading live feeds...</div>;
  if (error) return <div className="text-red-400">{error}</div>;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
      {/* DEX Scan Feed */}
      <div className="fusion-glass p-4 rounded-xl shadow-fusion-glow">
        <div className="text-xl font-fusion text-fusion-neon mb-2">DEX Scanning Feed</div>
        <div className="h-32 overflow-y-auto text-fusion text-sm">
          {dexFeed.map((event, i) => (
            <div key={i} className={`mb-1 ${event.status === 'error' ? 'text-red-400' : event.status === 'success' ? 'text-green-400' : ''}`}>
              [{new Date(event.timestamp).toLocaleTimeString()}] {event.dex}: {event.message}
            </div>
          ))}
        </div>
      </div>
      {/* Liquidation Feed */}
      <div className="fusion-glass p-4 rounded-xl shadow-fusion-glow">
        <div className="text-xl font-fusion text-fusion-neon mb-2">Liquidation Feed</div>
        <div className="h-32 overflow-y-auto text-fusion text-sm">
          {liqFeed.map((event, i) => (
            <div key={i} className={`mb-1 ${event.status === 'flagged' ? 'text-yellow-400' : event.status === 'liquidated' ? 'text-green-400' : 'text-gray-300'}`}>
              [{new Date(event.timestamp).toLocaleTimeString()}] {event.account} {event.status}: {event.details}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
