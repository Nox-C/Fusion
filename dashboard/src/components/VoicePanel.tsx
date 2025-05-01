// TypeScript: Extend window for speech API
declare global {
  interface Window {
    SpeechRecognition: any;
    webkitSpeechRecognition: any;
  }
}

import React, { useState, useEffect, useRef } from 'react';
import axios from 'axios';
import { useSpeechSynthesis } from 'react-speech-kit';

interface VoicePanelProps {
  onStartVoice: () => void;
  onEndVoice: () => void;
  isSpeaking: boolean;
}

interface VoiceStatus {
  isConnected: boolean;
  isProcessing: boolean;
  error?: string;
  lastMessage?: string;
}

// Type declarations for SpeechRecognition API
interface SpeechRecognitionResult {
  transcript: string;
  confidence: number;
}

interface SpeechRecognitionResultList {
  length: number;
  item: (index: number) => SpeechRecognitionResult | null;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionEvent extends Event {
  results: SpeechRecognitionResultList;
}

interface SpeechRecognition {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onstart: () => void;
  onresult: (event: SpeechRecognitionEvent) => void;
  onerror: (event: Event) => void;
  onend: () => void;
  start: () => void;
  stop: () => void;
}

const VOICE_NAME = 'Google UK English Female'; // fallback to best available female voice

function getFusionVoice(): SpeechSynthesisVoice | null {
  const voices = window.speechSynthesis.getVoices();
  return (
    voices.find(v => v.name === VOICE_NAME) ||
    voices.find(v => v.lang.startsWith('en') && v.name.toLowerCase().includes('female')) ||
    voices.find(v => v.lang.startsWith('en')) ||
    null
  );
}

export const VoicePanel: React.FC<VoicePanelProps> = ({ onStartVoice, onEndVoice, isSpeaking }) => {
  const [listening, setListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [aiReply, setAiReply] = useState('');
  const [status, setStatus] = useState<VoiceStatus>({
    isConnected: false,
    isProcessing: false,
    error: undefined,
    lastMessage: undefined,
  });
  const [speechSupport, setSpeechSupport] = useState({
    recognition: false,
    synthesis: false,
  });
  const recognitionRef = useRef<SpeechRecognition | null>(null);

  useEffect(() => {
    // Check for browser support
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    setSpeechSupport({
      recognition: !!SpeechRecognition,
      synthesis: !!window.speechSynthesis,
    });

    if (!SpeechRecognition) {
      setStatus(prev => ({
        ...prev,
        error: 'Speech recognition is not supported in your browser',
      }));
      return;
    }

    const recognition = new (SpeechRecognition as any)();
    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = 'en-US';
    recognitionRef.current = recognition;

    recognition.onstart = () => {
      setListening(true);
      setStatus(prev => ({
        ...prev,
        isConnected: true,
        isProcessing: true,
      }));
      onStartVoice();
    };

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      const transcript = Array.from(event.results)
        .map(result => result.transcript)
        .join('');
      setTranscript(transcript);
      setStatus(prev => ({
        ...prev,
        lastMessage: transcript,
      }));
    };

    recognition.onerror = (event) => {
      setStatus(prev => ({
        ...prev,
        error: event.error,
        isConnected: false,
        isProcessing: false,
      }));
    };

    recognition.onend = () => {
      setListening(false);
      setStatus(prev => ({
        ...prev,
        isProcessing: false,
      }));
      onEndVoice();
    };

    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
    };
  }, [onStartVoice, onEndVoice]);

  // Start speech recognition
  const startListening = () => {
    if (!('webkitSpeechRecognition' in window || 'SpeechRecognition' in window)) {
      alert('Speech recognition not supported in this browser.');
      return;
    }
    const SpeechRecognition = (window as any).webkitSpeechRecognition || (window as any).SpeechRecognition;
    const recognition = new SpeechRecognition();
    recognition.lang = 'en-US';
    recognition.interimResults = false;
    recognition.maxAlternatives = 1;
    recognition.onstart = () => {
      setListening(true);
      onStartVoice();
    };
    recognition.onresult = async (event) => {
      const transcript = event.results[0][0].transcript;
      setTranscript(transcript);
      
      // Send to Rust backend
      setStatus(prev => ({
        ...prev,
        isProcessing: true,
        lastMessage: transcript,
        error: undefined
      }));

      try {
        const response = await axios.post('/api/voice-process', { text: transcript });
        const data = response.data;
        setAiReply(data.reply);
        setStatus(prev => ({
          ...prev,
          isProcessing: false,
          error: undefined
        }));
      } catch (error: any) {
        console.error('Voice processing error:', error);
        setStatus(prev => ({
          ...prev,
          isProcessing: false,
          error: error.response?.data?.error || 'Failed to process request'
        }));
        setAiReply('Error: ' + (error.response?.data?.error || 'Failed to process request'));
      }
    };
    recognition.onend = () => {
      setListening(false);
      onEndVoice();
    };
    recognition.onerror = (event) => {
      console.error('Speech recognition error:', event.error);
      setListening(false);
      onEndVoice();
    };
    recognitionRef.current = recognition;
    setListening(true);
    onStartVoice();
    recognition.start();
    console.log('[SpeechRecognition] started');
  };

  // Fake AI backend for MVP
  function fakeFusionAI(text: string) {
    // Playful, aspirational, female FUSION personality
    let response = '';
    if (text.toLowerCase().includes('profit')) {
      response = "Profit is my passion! I'm optimizing every protocol for your maximum gain.";
    } else if (text.toLowerCase().includes('hello') || text.toLowerCase().includes('hi')) {
      response = "Hello! I'm FUSION, your AI partner. Ready to make history?";
    } else if (text.toLowerCase().includes('status')) {
      response = "All systems are nominal. Scan intervals and profits are being tuned in real time.";
    } else {
      response = "I'm here, always learning and evolving. What can I do for you next?";
    }
    setAiReply(response);
    speakFusion(response);
  }

  // Speak AI reply
  function speakFusion(text: string) {
    if (!window.speechSynthesis) {
      console.warn('[SpeechSynthesis] not supported');
      return;
    }
    const utter = new window.SpeechSynthesisUtterance(text);
    utter.rate = 1.05;
    utter.pitch = 1.15;
    utter.volume = 1;
    const fusionVoice = getFusionVoice();
    if (fusionVoice) utter.voice = fusionVoice;
    utter.onstart = () => {
      console.log('[SpeechSynthesis] start');
      onStartVoice();
    };
    utter.onend = () => {
      console.log('[SpeechSynthesis] end');
      onEndVoice();
    };
    utter.onerror = (err: any) => {
      console.error('[SpeechSynthesis] error:', err);
    };
    window.speechSynthesis.speak(utter);
    console.log('[SpeechSynthesis] speak:', text);
  }

  // Stop speech synthesis if needed
  useEffect(() => {
    return () => {
      window.speechSynthesis.cancel();
    };
  }, []);

  return (
    <div className="flex flex-col items-center justify-center w-full">
      {/* Visual feedback for voice states */}
      <div className="flex gap-2 mb-2">
        <span className={`w-3 h-3 rounded-full ${listening ? 'bg-yellow-400 animate-pulse' : 'bg-gray-700'}`} title="Listening"></span>
        <span className={`w-3 h-3 rounded-full ${transcript ? 'bg-green-400' : 'bg-gray-700'}`} title="Recognized"></span>
      </div>

      {/* Error Message */}
      {status.error && (
        <div className="text-red-400 text-sm mb-4">
          Error: {status.error}
        </div>
      )}

      <div className="flex items-center justify-center mb-4">
        <button
          onClick={startListening}
          disabled={!status.isConnected || status.isProcessing}
          className={`px-6 py-2 rounded-full text-lg font-semibold transition-all duration-300 ${
            listening ? 'bg-fusion-neon text-fusion-dark' : 'bg-fusion text-fusion-neon'
          } ${
            !status.isConnected || status.isProcessing ? 'opacity-50 cursor-not-allowed' : ''
          }`}
        >
          {listening ? 'Listening...' : status.isProcessing ? 'Processing...' : 'Speak'}
        </button>
      </div>

      {/* Voice Interaction History */}
      <div className="space-y-2">
        {status.lastMessage && (
          <div className="text-fusion text-sm">
            You: {status.lastMessage}
          </div>
        )}
        {aiReply && (
          <div className="text-fusion-neon text-sm">
            AI: {aiReply}
          </div>
        )}
      </div>
    </div>
  );
}
