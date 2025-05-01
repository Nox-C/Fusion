import { NextApiRequest, NextApiResponse } from 'next';
import fetch from 'node-fetch';
import { Request } from 'express';

// Extend NextApiRequest type to include method property
declare module 'next' {
  interface NextApiRequest {
    method: string;
  }
}

interface VoiceRequest {
  text: string;
}

interface VoiceResponse {
  reply?: string;
  status: 'success' | 'error' | 'processing';
  error?: string;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { text } = req.body as VoiceRequest;
  if (!text) {
    return res.status(400).json({ error: 'No text provided' });
  }

  try {
    // Connect to Rust backend
    const response = await fetch('http://localhost:8080/execute_arbitrage', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ command: text }),
    });

    if (!response.ok) {
      const errorData = await response.json() as VoiceResponse;
      throw new Error(errorData.error || 'Failed to process request');
    }

    const data = await response.json() as VoiceResponse;
    return res.status(200).json({
      reply: data.reply || 'Processing your request...',
      status: data.status || 'processing'
    });
  } catch (error) {
    console.error('Voice processing error:', error);
    res.status(500).json({
      reply: 'Error processing request',
      status: 'error'
    });
  }
}
