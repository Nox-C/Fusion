import { useRef, useEffect } from 'react';
import { useThree } from '@react-three/fiber';
import { AmbientLight, PointLight } from 'three';

export function Lights() {
  const { scene } = useThree();
  
  useEffect(() => {
    const ambient = new AmbientLight(0xffffff, 0.8);
    const point = new PointLight(0xffffff, 1, 10);
    point.position.set(10, 10, 10);
    
    scene.add(ambient);
    scene.add(point);
    
    return () => {
      scene.remove(ambient);
      scene.remove(point);
    };
  }, [scene]);
  
  return null;
}
