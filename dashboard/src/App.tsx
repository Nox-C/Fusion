import React, { useRef, useState } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Lights } from './components/Lights';
import FusionOrb from './components/FusionOrb';
import FusionFace from './components/FusionFace';
import { VoicePanel } from './components/VoicePanel';
import DashboardStats from './components/DashboardStats';
import { LiveFeeds } from './components/LiveFeeds';

export default function App() {
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [faceMode, setFaceMode] = useState(false);

  // When user or AI initiates communication, morph orb to face
  const handleStartVoice = () => {
    setFaceMode(true);
    setIsSpeaking(true);
  };
  const handleEndVoice = () => {
    setIsSpeaking(false);
    setTimeout(() => setFaceMode(false), 1500); // morph back after a pause
  };

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
      <div className="w-full max-w-6xl px-4 mt-10 z-20">
        <DashboardStats />
        <LiveFeeds />
      </div>
    </div>
  );
}
