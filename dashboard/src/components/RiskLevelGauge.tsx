import React from 'react';
import GaugeChart from 'react-gauge-chart';

interface RiskLevelGaugeProps {
  currentLevel: number;
}

export const RiskLevelGauge: React.FC<RiskLevelGaugeProps> = ({ currentLevel }) => {
  const normalizedLevel = Math.min(100, Math.max(0, currentLevel * 10)); // Convert 1-10 to 0-100

  return (
    <div className="bg-fusion-dark p-4 rounded-lg">
      <h3 className="text-fusion-accent mb-2">Risk Level</h3>
      <div className="flex flex-col items-center">
        <GaugeChart
          id="risk-gauge"
          nrOfLevels={10}
          colors={['#44ff44', '#ff8800', '#ff4444']}
          arcWidth={0.2}
          percent={normalizedLevel / 100}
          style={{ width: '200px', height: '200px' }}
        />
        <div className="text-fusion-neon text-xl mt-2">
          {currentLevel.toFixed(0)}
        </div>
      </div>
    </div>
  );
};
