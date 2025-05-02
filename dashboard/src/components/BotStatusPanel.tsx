import React from 'react';

interface BotStatusPanelProps {
  statuses: Record<string, { status: string; message?: string }>;
}

export const BotStatusPanel: React.FC<BotStatusPanelProps> = ({ statuses }) => {
  return (
    <div className="bg-fusion-dark p-4 rounded-lg">
      <h3 className="text-fusion-accent mb-2">Bot Statuses</h3>
      <div className="space-y-2">
        {Object.entries(statuses).map(([botName, { status, message }]) => (
          <div key={botName} className="flex items-center justify-between">
            <div className="flex-1 min-w-0">
              <div className="text-fusion-neon font-medium truncate">{botName}</div>
              <div className="text-xs text-gray-400 truncate">{message || 'No message'}</div>
            </div>
            <div className={`px-2 py-1 rounded-full text-xs font-medium ${
              status === 'Idle' ? 'bg-green-500/20 text-green-400' :
              status === 'Scanning' ? 'bg-blue-500/20 text-blue-400' :
              'bg-red-500/20 text-red-400'
            }`}>
              {status}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
