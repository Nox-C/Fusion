// TypeScript: Extend window for speech API
interface Window {
  SpeechRecognition: any;
  webkitSpeechRecognition: any;
}

import React, { useEffect, useRef, useState } from 'react';

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

export default function VoicePanel({ onStartVoice, onEndVoice, isSpeaking }: {
  onStartVoice: () => void;
  onEndVoice: () => void;
  isSpeaking: boolean;
}) {
  const [listening, setListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [aiReply, setAiReply] = useState('');
  const [speechSupport, setSpeechSupport] = useState({ recognition: true, synthesis: true });
  const recognitionRef = useRef<any>(null);

  useEffect(() => {
    // Check for browser support
    setSpeechSupport({
      recognition: !!(window.SpeechRecognition || window.webkitSpeechRecognition),
      synthesis: !!window.speechSynthesis,
    });
  }, []);

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
      console.log('[SpeechRecognition] start');
    };
    recognition.onresult = (event: any) => {
      const text = event.results[0][0].transcript;
      setTranscript(text);
      setListening(false);
      onEndVoice();
      console.log('[SpeechRecognition] result:', text);
      // Send to AI backend (placeholder)
      fakeFusionAI(text);
    };
    recognition.onend = () => {
      setListening(false);
      onEndVoice();
      console.log('[SpeechRecognition] end');
    };
    recognition.onerror = (err: any) => {
      setListening(false);
      onEndVoice();
      console.error('[SpeechRecognition] error:', err);
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
        <span className={`w-3 h-3 rounded-full ${isSpeaking ? 'bg-cyan-400 animate-pulse' : 'bg-gray-700'}`} title="Speaking"></span>
      </div>
      {/* Speech support warning */}
      {(!speechSupport.recognition || !speechSupport.synthesis) && (
        <div className="text-red-400 text-sm mb-2">⚠️ Voice features not supported in this browser.</div>
      )}
      <div className="mt-2 text-fusion-neon text-lg min-h-[1.5em]">{transcript}</div>
      <div className="mt-2 text-fusion text-lg min-h-[2em]">{aiReply}</div>
      <div className="w-full flex justify-center mt-6">
        <button
          className={`rounded-full px-8 py-4 bg-fusion-neon text-fusion-dark text-xl font-fusion shadow-fusion-glow transition-all ${listening ? 'scale-110' : ''}`}
          onClick={startListening}
          disabled={listening || isSpeaking}
          aria-label="Speak to FUSION"
        >
          {listening ? 'Listening...' : 'Talk to FUSION'}
        </button>
      </div>
    </div>
  );
}
