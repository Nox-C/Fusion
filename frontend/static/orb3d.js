// Orb3D.js - Simple Three.js 3D Floating Orb
import * as THREE from 'https://cdn.jsdelivr.net/npm/three@0.152.2/build/three.module.js';

export function createOrb3D(containerId = 'orb3d-canvas') {
    let width = 220, height = 220;
    const container = document.getElementById(containerId);
    if (!container) return;
    container.innerHTML = '';
    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setSize(width, height);
    renderer.setClearColor(0x000000, 0);
    container.appendChild(renderer.domElement);

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(38, width / height, 0.1, 1000);
    camera.position.z = 60;

    // Orb geometry
    const geometry = new THREE.SphereGeometry(32, 64, 64);
    const material = new THREE.MeshPhysicalMaterial({
        color: 0x00fff7,
        metalness: 0.7,
        roughness: 0.12,
        transmission: 0.93,
        thickness: 6.2,
        clearcoat: 1.0,
        clearcoatRoughness: 0.04,
        ior: 1.4,
        envMapIntensity: 1.2,
        emissive: 0xae00ff,
        emissiveIntensity: 0.25,
    });
    const orb = new THREE.Mesh(geometry, material);
    scene.add(orb);

    // Lights
    const ambient = new THREE.AmbientLight(0x00fff7, 0.66);
    scene.add(ambient);
    const point = new THREE.PointLight(0xae00ff, 2.7, 300);
    point.position.set(30, 35, 60);
    scene.add(point);
    const back = new THREE.PointLight(0x00fff7, 1.3, 200);
    back.position.set(-30, -50, -50);
    scene.add(back);

    // Animate
    function animate() {
        requestAnimationFrame(animate);
        orb.rotation.y += 0.006;
        orb.rotation.x += 0.002;
        orb.position.y = Math.sin(Date.now() * 0.001) * 4;
        renderer.render(scene, camera);
    }
    animate();
}

// Auto-initialize if a div#orb3d-canvas is present
window.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById('orb3d-canvas')) {
        createOrb3D('orb3d-canvas');
    }
});

window.createOrb3D = createOrb3D;
