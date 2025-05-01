import React, { useState, useEffect, useRef, useCallback } from 'react';
import { z } from 'zod';
import { WebSocket, MessageEvent } from 'ws';

// Define zod schemas for validation
const dexScanEventSchema = z.object({
  type: z.literal('dex'),
  id: z.string(),
  timestamp: z.string().datetime(),
  dex: z.string(),
  message: z.string(),
  status: z.string()
});

const liquidationEventSchema = z.object({
  id: z.string(),
  type: z.literal('liquidation'),
  timestamp: z.string().datetime(),
  account: z.string().startsWith('0x'), // Ensure it's a valid Ethereum address
  status: z.string(),
  details: z.string()
});

// TypeScript types derived from schemas
export type DexScanEvent = z.infer<typeof dexScanEventSchema>;
export type LiquidationEvent = z.infer<typeof liquidationEventSchema>;

const MAX_DEX_FEED_LENGTH = 100; // Limit the number of DEX events stored
const MAX_LIQUIDATION_FEED_LENGTH = 100; // Limit the number of liquidation events stored

export const LiveFeeds: React.FC = () => {
  const [dexFeed, setDexFeed] = useState<DexScanEvent[]>([]);
  const [liquidationFeed, setLiquidationFeed] = useState<LiquidationEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState(false); // Track connection status

  // useRef is better for managing mutable objects like WebSocket that don't need to trigger re-renders on change.
  useEffect(() => {
    const wsRef = useRef<WebSocket | null>(null);
    const retryTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const retryDelayRef = useRef(1000); // Start with 1 second delay

    // useCallback ensures this function reference is stable across re-renders unless dependencies change.
    // It's good practice, though less critical here as the dependency array is empty.
    const connectWebSocket = useCallback(() => {
      // Clear any pending retry timer before attempting connection
      if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current);
        retryTimeoutRef.current = null;
      }

      // Prevent multiple connections if one is already open or connecting
      if (wsRef.current && (wsRef.current.readyState === WebSocket.OPEN || wsRef.current.readyState === WebSocket.CONNECTING)) {
        console.log('WebSocket already open or connecting.');
        return;
      }

      // Ensure previous connection is properly closed before creating a new one
      if (wsRef.current) {
        console.log('Closing previous WebSocket connection before reconnecting.');
        wsRef.current.onclose = null; // Remove old handlers to prevent interference
        wsRef.current.onerror = null;
        wsRef.current.onmessage = null;
        if (wsRef.current) {
          wsRef.current.close();
          wsRef.current = null;
        }
      }

      try {
        const socketUrl = 'ws://localhost:8080/ws/matrix2d';
        wsRef.current = new WebSocket(socketUrl);
        console.log(`Attempting WebSocket connection to ${socketUrl}`);
        setLoading(true); // Show loading indicator during connection attempt
        setError(null);   // Clear previous errors
        setIsConnected(false); // Set connection status to false initially

        wsRef.current.onopen = () => {
          console.log('WebSocket connection established successfully');
          setLoading(false);
          setError(null);
          setIsConnected(true);
          retryDelayRef.current = 1000; // Reset retry delay on successful connection
        };

        const handleWebSocketMessage = (event: import('ws').MessageEvent) => {
          console.log('Received WebSocket message:', event.data);
          try {
            const data = event.data instanceof ArrayBuffer ? String.fromCharCode.apply(null, new Uint8Array(event.data)) : event.data.toString();
            
            // Validate DEX event
            if (data.type === 'dex') {
              try {
                const validatedEvent = dexScanEventSchema.parse(JSON.parse(data));
                setDexFeed(prev => [validatedEvent, ...prev].slice(0, MAX_DEX_FEED_LENGTH));
              } catch (error) {
                console.warn('Invalid DEX event:', error);
                // Optionally log the invalid data for debugging
                console.log('Invalid DEX event data:', data);
              }
            }
            // Validate liquidation event
            else if (data.type === 'liquidation') {
              try {
                const validatedEvent = liquidationEventSchema.parse(data);
                setLiquidationFeed(prev => [validatedEvent, ...prev].slice(0, MAX_LIQUIDATION_FEED_LENGTH));
              } catch (error) {
                console.warn('Invalid liquidation event:', error);
                // Optionally log the invalid data for debugging
                console.log('Invalid liquidation event data:', data);
              }
            }
            else {
              console.warn('Received unknown message type:', data);
            }
          } catch (parseError) {
            console.error('Failed to parse WebSocket message:', parseError, 'Raw data:', event.data);
            // Optionally set an error state here if needed
          }
        };

        const handleWebSocketError = (event: import('ws').ErrorEvent) => {
          handleConnectionLoss(`WebSocket error: ${event.type}`);
        };

        const handleWebSocketClose = (event: import('ws').CloseEvent) => {
          // Only retry if the close was unexpected (not initiated by cleanup)
          if (!event.wasClean) {
            handleConnectionLoss(`WebSocket closed unexpectedly (Code: ${event.code}, Reason: ${event.reason || 'N/A'})`);
          } else {
            console.log('WebSocket closed cleanly.');
            setIsConnected(false);
            // Optionally set loading/error state if a clean close is unexpected during operation
          }
        };

        wsRef.current.onmessage = handleWebSocketMessage;
        wsRef.current.onerror = handleWebSocketError;
        wsRef.current.onclose = handleWebSocketClose;

        // Shared error/close handling logic for retries
        const handleConnectionLoss = (reason: string) => {
          console.error(reason);
          if (wsRef.current && wsRef.current.readyState !== WebSocket.CLOSED) {
            wsRef.current.close(); // Ensure socket is closed
          }
          wsRef.current = null; // Nullify the ref
          setIsConnected(false);
          setLoading(false); // Stop loading indicator, show error instead
          setError(`${reason}. Retrying...`);

          // Exponential backoff retry
          const currentDelay = retryDelayRef.current;
          retryTimeoutRef.current = setTimeout(() => {
            console.log(`Retrying connection after ${retryDelayRef.current}ms delay...`);
            connectWebSocket();
          }, retryDelayRef.current);
          // Increase delay for next time, capped at 30 seconds
          retryDelayRef.current = Math.min(retryDelayRef.current * 2, 30000);
        };

      } catch (error) {
        console.error('Failed to instantiate WebSocket:', error);
        setError('Failed to create WebSocket connection. Check console.');
        setLoading(false); // Stop loading, show error
        setIsConnected(false);
        // Still attempt retry even if constructor fails (e.g., invalid URL initially)
        const currentDelay = retryDelayRef.current;
        retryTimeoutRef.current = setTimeout(() => {
          console.log(`Retrying connection after ${retryDelayRef.current}ms delay...`);
          connectWebSocket();
        }, currentDelay);
        retryDelayRef.current = Math.min(currentDelay * 2, 30000);
      }
    }, []); // Empty dependency array means connectWebSocket is created once

    connectWebSocket();

    // Cleanup function: runs when the component unmounts
    return () => {
      console.log('LiveFeeds component unmounting. Cleaning up...');
      // Clear any pending retry timeout
      if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current);
        console.log('Cleared pending retry timeout.');
      }
      // Close the WebSocket connection cleanly
      if (wsRef.current) {
        console.log('Cleaning up WebSocket connection');
        wsRef.current.onclose = null;
        wsRef.current.onerror = null;
        wsRef.current.onmessage = null;
        wsRef.current.close(1000, "Component unmounted"); // 1000 indicates Normal Closure
      }
    };
  }, []); // No dependencies needed

  // Display loading, error, or connection status
  const statusMessage = loading ? (
    <div className="text-fusion-neon">Connecting to live feeds...</div>
  ) : error ? (
    <div className="text-red-400">{error}</div>
  ) : !isConnected ? (
    <div className="text-yellow-400">WebSocket disconnected. Attempting to reconnect...</div>
  ) : null;

  return (
    <div className="space-y-4">
      {statusMessage /* Display status message at the top */}
      <div className="bg-fusion-dark p-4 rounded-lg">
        <h3 className="text-fusion-neon mb-2">DEX Scans</h3>
        <div className="space-y-2 h-64 overflow-y-auto"> {/* Added height and scroll */}
          {dexFeed.length === 0 && !loading && !error && <div className="text-gray-500">No DEX scan events yet...</div>}
          {/* Using timestamp + dex as a key. If events can have identical timestamps+dex, a unique ID from backend is better */}
          {dexFeed.map((event) => (
            <div key={event.id} className="text-fusion-neon">
              <div className="font-semibold">{event.dex}</div>
              <div>{event.message}</div>
              <div className="text-sm opacity-75">{new Date(event.timestamp).toLocaleString()}</div>
            </div>
          ))}
        </div>
      </div>

      <div className="bg-fusion-dark p-4 rounded-lg">
        <h3 className="text-fusion-neon mb-2">Liquidations</h3>
        <div className="space-y-2 h-64 overflow-y-auto"> {/* Added height and scroll */}
          {liquidationFeed.length === 0 && !loading && !error && <div className="text-gray-500">No liquidation events yet...</div>}
          {/* Using timestamp + account as a key. If events can have identical timestamps+accounts, a unique ID from backend is better */}
          {liquidationFeed.map((event) => (
            <div key={event.id} className="text-fusion-neon">
              <div className="font-semibold">{event.account}</div>
              <div>{event.details}</div>
              <div className="text-sm opacity-75">{new Date(event.timestamp).toLocaleString()}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
