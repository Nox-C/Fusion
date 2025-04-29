import React, { useRef, useState } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Html } from '@react-three/drei';
import FusionOrb from './components/FusionOrb';
import FusionFace from './components/FusionFace';
import VoicePanel from './components/VoicePanel';

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
    <div className="min-h-screen flex flex-col items-center justify-center bg-gradient-to-br from-fusion-dark to-fusion-neon">
      <div className="relative w-full h-[60vh] flex items-center justify-center">
        <Canvas camera={{ position: [0, 0, 4] }} className="absolute inset-0">
          <ambientLight intensity={0.8} />
          <pointLight position={[10, 10, 10]} />
          {!faceMode && <FusionOrb />}
          {faceMode && <FusionFace speaking={isSpeaking} />}
          <OrbitControls enablePan={false} enableZoom={false} />
        </Canvas>
        <Html center>
          <VoicePanel
            onStartVoice={handleStartVoice}
            onEndVoice={handleEndVoice}
            isSpeaking={isSpeaking}
          />
        </Html>
      </div>
      <div className="mt-8 w-full max-w-4xl fusion-glass p-8">
        {/* System stats, graphs, and gauges will go here */}
        <h1 className="text-4xl font-fusion text-fusion-neon mb-2">FUSION Dashboard</h1>
        <p className="text-lg text-fusion">Welcome! Speak or click the orb to interact with FUSION.</p>
      </div>
    </div>
  );
}
