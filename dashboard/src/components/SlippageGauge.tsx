import React from 'react';
import GaugeChart from 'react-gauge-chart';

interface SlippageGaugeProps {
  currentSlippage: number;
}

export const SlippageGauge: React.FC<SlippageGaugeProps> = ({ currentSlippage }) => {
  const normalizedSlippage = Math.min(100, Math.max(0, currentSlippage * 100)); // Convert to percentage

  return (
    <div className="bg-fusion-dark p-4 rounded-lg">
      <h3 className="text-fusion-accent mb-2">Slippage Tolerance</h3>
      <div className="flex flex-col items-center">
        <GaugeChart
          id="slippage-gauge"
          nrOfLevels={10}
          colors={['#44ff44', '#ff8800', '#ff4444']}
          arcWidth={0.2}
          percent={normalizedSlippage / 100}
          style={{ width: '200px', height: '200px' }}
        />
        <div className="text-fusion-neon text-xl mt-2">
          {currentSlippage.toFixed(2)}%
        </div>
      </div>
    </div>
  );
};
