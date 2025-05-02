import React from 'react';
import GaugeChart from 'react-gauge-chart';

interface GasGaugeProps {
  currentGasPrice: number;
}

export const GasGauge: React.FC<GasGaugeProps> = ({ currentGasPrice }) => {
  const normalizedGas = Math.min(100, Math.max(0, currentGasPrice / 200 * 100)); // Normalize to 0-100

  return (
    <div className="bg-fusion-dark p-4 rounded-lg">
      <h3 className="text-fusion-accent mb-2">Gas Price</h3>
      <div className="flex flex-col items-center">
        <GaugeChart
          id="gas-gauge"
          nrOfLevels={20}
          colors={['#ff4444', '#ff8800', '#44ff44']}
          arcWidth={0.2}
          percent={normalizedGas / 100}
          style={{ width: '200px', height: '200px' }}
        />
        <div className="text-fusion-neon text-xl mt-2">
          {currentGasPrice.toFixed(1)} Gwei
        </div>
      </div>
    </div>
  );
};
