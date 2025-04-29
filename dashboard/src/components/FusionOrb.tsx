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
    <mesh ref={mesh} castShadow receiveShadow>
      <sphereGeometry args={[1, 64, 64]} />
      <meshPhysicalMaterial
        color="#b8c6db"
        metalness={1}
        roughness={0.2}
        clearcoat={1}
        clearcoatRoughness={0.1}
        reflectivity={1}
        iridescence={0.6}
        transmission={0.5}
        thickness={1}
        envMapIntensity={1.2}
      />
    </mesh>
  );
}
