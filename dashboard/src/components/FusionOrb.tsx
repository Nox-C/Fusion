import React, { useRef } from 'react';
import { useFrame } from '@react-three/fiber';

export default function FusionOrb() {
  const mesh = useRef<any>();
  useFrame((state) => {
    if (mesh.current) {
      mesh.current.position.y = Math.sin(state.clock.getElapsedTime()) * 0.15;
      mesh.current.rotation.y += 0.005;
    }
  });
  return (
    <>
      {/* Main orb */}
      <mesh ref={mesh} castShadow receiveShadow>
        <sphereGeometry args={[1, 64, 64]} />
        <meshPhysicalMaterial
          color="#b8c6db"
          metalness={1}
          roughness={0.1}
          clearcoat={1}
          clearcoatRoughness={0.05}
          reflectivity={1}
          iridescence={0.8}
          transmission={0.7}
          thickness={2}
          envMapIntensity={2.5}
          emissive="#00fff7"
          emissiveIntensity={1.5}
        />
      </mesh>
      {/* Neon glow shell */}
      <mesh ref={mesh} scale={[1.08, 1.08, 1.08]}>
        <sphereGeometry args={[1, 64, 64]} />
        <meshBasicMaterial color="#00fff7" transparent opacity={0.25} />
      </mesh>
      {/* Extra point light for glow */}
      <pointLight position={[0, 0, 2]} intensity={1.2} color="#00fff7" distance={6} />
    </>
  );
}
