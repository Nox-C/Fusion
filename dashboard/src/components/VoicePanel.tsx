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
  const recognitionRef = useRef<any>(null);

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
    recognition.onresult = (event: any) => {
      const text = event.results[0][0].transcript;
      setTranscript(text);
      setListening(false);
      onEndVoice();
      // Send to AI backend (placeholder)
      fakeFusionAI(text);
    };
    recognition.onend = () => {
      setListening(false);
      onEndVoice();
    };
    recognition.onerror = () => {
      setListening(false);
      onEndVoice();
    };
    recognitionRef.current = recognition;
    setListening(true);
    onStartVoice();
    recognition.start();
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
    const utter = new window.SpeechSynthesisUtterance(text);
    utter.rate = 1.05;
    utter.pitch = 1.15;
    utter.volume = 1;
    const fusionVoice = getFusionVoice();
    if (fusionVoice) utter.voice = fusionVoice;
    utter.onstart = () => onStartVoice();
    utter.onend = () => onEndVoice();
    window.speechSynthesis.speak(utter);
  }

  // Stop speech synthesis if needed
  useEffect(() => {
    return () => {
      window.speechSynthesis.cancel();
    };
  }, []);

  return (
    <div className="flex flex-col items-center justify-center">
      <button
        className={`rounded-full px-8 py-4 bg-fusion-neon text-fusion-dark text-xl font-fusion shadow-fusion-glow transition-all ${listening ? 'scale-110' : ''}`}
        onClick={startListening}
        disabled={listening || isSpeaking}
        aria-label="Speak to FUSION"
      >
        {listening ? 'Listening...' : 'Talk to FUSION'}
      </button>
      <div className="mt-2 text-fusion-neon text-lg min-h-[1.5em]">{transcript}</div>
      <div className="mt-2 text-fusion text-lg min-h-[2em]">{aiReply}</div>
    </div>
  );
}
