import React, { useEffect, useState } from 'react';
import { ProtocolStat } from '../api/types';
import { fetchDashboardData } from '../api/mock';

export default function DashboardStats() {
  const [stats, setStats] = useState<ProtocolStat[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchDashboardData()
      .then(data => {
        setStats(data.protocol_stats);
        setLoading(false);
      })
      .catch(err => {
        setError('Failed to load stats');
        setLoading(false);
      });
  }, []);

  if (loading) return <div className="text-fusion-neon">Loading stats...</div>;
  if (error) return <div className="text-red-400">{error}</div>;

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      {stats.map((stat) => (
        <div key={stat.name} className="fusion-glass p-6 rounded-xl flex flex-col items-center shadow-fusion-glow">
          <div className="text-2xl font-fusion text-fusion-neon mb-2">{stat.name}</div>
          <div className="text-4xl font-bold">{stat.profit_threshold}</div>
          <div className="text-fusion mt-1">Scan: {stat.scan_interval}s</div>
          <div className="text-fusion mt-1">Weight: {stat.weight}</div>
          <div className={`mt-1 text-xs ${stat.ai_tuned ? 'text-fusion-neon' : 'text-gray-400'}`}>{stat.ai_tuned ? 'AI Tuned' : 'Manual'}</div>
        </div>
      ))}
    </div>
  );
}
