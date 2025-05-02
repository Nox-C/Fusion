import React, { useRef, useState, useEffect, useCallback } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { z } from 'zod'; // Import Zod
import { Lights } from './components/Lights';
import FusionOrb from './components/FusionOrb';
import FusionFace from './components/FusionFace';
import { VoicePanel } from './components/VoicePanel';
import DashboardStats from './components/DashboardStats';
import { GasGauge } from './components/GasGauge';
import { LiveFeeds } from './components/LiveFeeds';
import { SlippageGauge } from './components/SlippageGauge';
import { RiskLevelGauge } from './components/RiskLevelGauge';
import { BotStatusPanel } from './components/BotStatusPanel';
import { ProtocolStat, DexScanEvent, LiquidationEvent, OpportunityMetric, ParameterUpdateEvent, BotStatusEvent } from './api/types';

// Constants for feed length limits
const MAX_DEX_FEED_LENGTH = 100;
const MAX_LIQUIDATION_FEED_LENGTH = 100;

// WebSocket configuration constants
const MAX_RETRIES = 5;
const MAX_RETRY_DELAY = 30000; // 30 seconds max delay

// Zod schemas for WebSocket events - using existing types
const parameterUpdateSchema = z.object({
  type: z.literal('parameter_update'),
  parameter_name: z.string(),
  new_value: z.string(),
  timestamp: z.string().datetime(),
  source: z.string(),
});

const botStatusSchema = z.object({
  type: z.literal('bot_status'),
  bot_name: z.string(),
  status: z.string(),
  message: z.string().optional(),
  timestamp: z.string().datetime(),
});

// Using existing types from api/types instead of defining new schemas
const dexScanEventSchema = z.object({
  type: z.literal('dex'),
  timestamp: z.string().datetime(),
  dex: z.string(),
  status: z.enum(['scanning', 'success', 'error']),
  message: z.string(),
});

const liquidationEventSchema = z.object({
  type: z.literal('liquidation'),
  timestamp: z.string().datetime(),
  account: z.string().startsWith('0x'),
  status: z.enum(['flagged', 'liquidated', 'healthy']),
  details: z.string(),
});

const webSocketEventSchema = z.union([
  dexScanEventSchema,
  liquidationEventSchema,
  parameterUpdateSchema,
  botStatusSchema,
]);

export default function App() {
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [faceMode, setFaceMode] = useState(false);
  // --- Lifted State ---
  const [gasPrice, setGasPrice] = useState(50); // Initial value in Gwei
  const [slippageTolerance, setSlippageTolerance] = useState(0.5);
  const [riskLevel, setRiskLevel] = useState(5);
  const [botStatuses, setBotStatuses] = useState<Record<string, { status: string; message?: string }>>({});
  const [dexFeed, setDexFeed] = useState<DexScanEvent[]>([]);
  const [liquidationFeed, setLiquidationFeed] = useState<LiquidationEvent[]>([]);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const retryTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const retryCountRef = useRef(0);

  // When user or AI initiates communication, morph orb to face
  // --- Placeholder for gas price state ---
  // In a real app, update this via WebSocket or API fetch in useEffect
  const handleStartVoice = () => {
    setFaceMode(true);
    setIsSpeaking(true);
  };
  const handleEndVoice = () => {
    setIsSpeaking(false);
    setTimeout(() => setFaceMode(false), 1500); // morph back after a pause
  };

  // --- Event Handling ---
  const handleWebSocketMessage = useCallback((event: MessageEvent) => {
    try {
      const jsonData = JSON.parse(event.data);
      const result = webSocketEventSchema.safeParse(jsonData);

      if (result.success) {
        const eventData = result.data;
        switch (eventData.type) {
          case 'dex':
            const dexEvent = eventData as DexScanEvent;
            setDexFeed(prev => [dexEvent, ...prev].slice(0, MAX_DEX_FEED_LENGTH));
            break;
          case 'liquidation':
            const liquidationEvent = eventData as LiquidationEvent;
            setLiquidationFeed(prev => [liquidationEvent, ...prev].slice(0, MAX_LIQUIDATION_FEED_LENGTH));
            break;
          case 'parameter_update':
            const updateEvent = eventData as ParameterUpdateEvent;
            if (updateEvent.parameter_name === 'slippage_tolerance') {
              setSlippageTolerance(parseFloat(updateEvent.new_value));
            } else if (updateEvent.parameter_name === 'risk_level') {
              setRiskLevel(parseInt(updateEvent.new_value, 10));
            }
            break;
          case 'bot_status':
            const statusEvent = eventData as BotStatusEvent;
            setBotStatuses(prev => ({
              ...prev,
              [statusEvent.bot_name]: {
                status: statusEvent.status,
                message: statusEvent.message
              }
            }));
            break;
        }
      } else {
        console.error('Invalid WebSocket message:', result.error);
        setError('Invalid message format');
      }
    } catch (error) {
      console.error('Error processing WebSocket message:', error);
      setError('Error processing message');
    }
  }, []);

  // --- Connection Management ---
  const wsRef = useRef<WebSocket | null>(null);

  const connectWebSocket = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    const socket = new WebSocket('ws://localhost:3001');
    wsRef.current = socket;
    setLoading(true);
    setError(null);
    setIsConnected(false);
    retryCountRef.current = 0;

    socket.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
      setLoading(false);
    };

    socket.onmessage = handleWebSocketMessage;

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setError('WebSocket error occurred');
    };

    socket.onclose = (event) => {
      console.log('WebSocket closed:', event.code, event.reason);
      setIsConnected(false);
      setLoading(false);
      retryWebSocketConnection();
    };
  }, [handleWebSocketMessage]);

  const retryWebSocketConnection = useCallback(() => {
    retryCountRef.current++;
    const backoff = Math.min(30000, 1000 * Math.pow(2, retryCountRef.current)); // Exponential backoff with max 30s
    const jitter = Math.random() * (backoff * 0.2); // Add 20% jitter

    console.log(`Retrying connection in ${Math.round((backoff + jitter) / 1000)}s...`);

    retryTimeoutRef.current = setTimeout(connectWebSocket, backoff + jitter);
  }, [connectWebSocket]);

  useEffect(() => {
    connectWebSocket();

    return () => {
      if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connectWebSocket]);

  return (
    <div className="min-h-screen flex flex-col items-center justify-start pt-8 relative">
      {/* Matrix rain overlay for digital depth */}
      <div className="matrix-rain" aria-hidden="true"></div>
      {/* 3D Orb/Face morph section */}
      <div className="relative w-full h-[60vh] flex items-center justify-center z-10" style={{perspective: 1200}}>
        {/* 3D Canvas for orb/face morph */}
        <Canvas camera={{ position: [0, 0, 4] }} className="absolute inset-0">
          <Lights />
          {!faceMode && <FusionOrb />}
          {faceMode && <FusionFace speaking={isSpeaking} />}
          <OrbitControls enablePan={false} enableZoom={false} />
        </Canvas>
        {/* Orb shadow for depth */}
        <div className="fusion-orb-shadow" style={{top: '70%'}}></div>
        {/* Fallback orb if Three.js fails completely */}
        <noscript>
          <div className="fusion-orb w-32 h-32 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"></div>
        </noscript>
      </div>
      {/* VoicePanel below orb/face */}
      <div className="w-full flex justify-center mt-6 z-20">
        <VoicePanel
          onStartVoice={handleStartVoice}
          onEndVoice={handleEndVoice}
          isSpeaking={isSpeaking}
        />
      </div>
      {/* Dashboard stats and live feeds, visually layered above background */}
      {/* Adjusted Grid Layout: Gauges/Status (Left), Feeds (Center), Stats (Right) */}
      <div className="w-full max-w-7xl px-4 mt-8 z-20 grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* --- Column 1: AI Gauges & Bot Status --- */}
        <div className="md:col-span-1 space-y-4">
           <h2 className="text-fusion-accent text-lg font-light tracking-wider mb-2">AI Control</h2>
           <RiskLevelGauge currentLevel={riskLevel} />
           <SlippageGauge currentSlippage={slippageTolerance} />
           <BotStatusPanel statuses={botStatuses} />
        </div>
        {/* --- Column 2: Live Feeds --- */}
        <div className="lg:col-span-2">
           <LiveFeeds
  dexFeed={dexFeed}
  liquidationFeed={liquidationFeed}
/>        </div>
        {/* --- Column 3: General Stats --- */}
        <div className="lg:col-span-1 space-y-4">
           <h2 className="text-fusion-accent text-lg font-light tracking-wider mb-2">Market Info</h2>
            <GasGauge currentGasPrice={gasPrice} />
           <DashboardStats /> {/* Assuming this shows general stats */}
        </div>
      </div>
    </div>
  );
}
