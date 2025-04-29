import React, { useRef } from 'react';
import { useFrame } from '@react-three/fiber';

// Placeholder: stylized, minimalistic, animated female face
// (For MVP: use simple geometry and color; can be replaced with morph targets or 3D model later)
export default function FusionFace({ speaking }: { speaking: boolean }) {
  const mouth = useRef<any>();
  useFrame((state) => {
    if (mouth.current && speaking) {
      mouth.current.scale.y = 1 + Math.abs(Math.sin(state.clock.getElapsedTime() * 6)) * 0.6;
    } else if (mouth.current) {
      mouth.current.scale.y = 1;
    }
  });
  return (
    <group>
      {/* Head */}
      <mesh position={[0, 0, 0]}>
        <sphereGeometry args={[1, 64, 64]} />
        <meshPhysicalMaterial color="#b8c6db" metalness={1} roughness={0.2} clearcoat={1} />
      </mesh>
      {/* Eyes */}
      <mesh position={[-0.35, 0.25, 0.95]}>
        <sphereGeometry args={[0.09, 32, 32]} />
        <meshStandardMaterial color="#00fff7" emissive="#00fff7" emissiveIntensity={0.5} />
      </mesh>
      <mesh position={[0.35, 0.25, 0.95]}>
        <sphereGeometry args={[0.09, 32, 32]} />
        <meshStandardMaterial color="#00fff7" emissive="#00fff7" emissiveIntensity={0.5} />
      </mesh>
      {/* Mouth */}
      <mesh ref={mouth} position={[0, -0.25, 0.98]} scale={[0.25, 1, 0.1]}>
        <sphereGeometry args={[0.12, 32, 32]} />
        <meshStandardMaterial color="#00fff7" emissive="#00fff7" emissiveIntensity={0.4} />
      </mesh>
    </group>
  );
}
