// Liquid Chrome Neon Morphing Orb Animation
// This script animates a morphing, liquid chrome/neon orb on a canvas.

const canvas = document.getElementById('liquid-orb-canvas');
if (canvas) {
    const ctx = canvas.getContext('2d');
    let width = canvas.width = window.innerWidth;
    let height = canvas.height = window.innerHeight;
    let t = 0;
    const orbRadius = Math.min(width, height) * 0.18;
    const orbCenter = { x: width/2, y: height/2 };
    const points = 24;
    const baseNoise = [];
    for (let i = 0; i < points; i++) baseNoise.push(Math.random());

    function draw() {
        ctx.clearRect(0, 0, width, height);
        ctx.save();
        ctx.translate(orbCenter.x, orbCenter.y);
        ctx.beginPath();
        for (let i = 0; i <= points; i++) {
            let angle = (i / points) * Math.PI * 2;
            let noise = baseNoise[i % points] + 0.25 * Math.sin(t + i);
            let morph = orbRadius + Math.sin(t + i * 0.3) * 20 + Math.cos(t * 0.7 + i) * 12 + noise * 18;
            let x = Math.cos(angle) * morph;
            let y = Math.sin(angle) * morph;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        }
        ctx.closePath();
        // Chrome/Neon gradient
        let grad = ctx.createRadialGradient(0, 0, orbRadius * 0.25, 0, 0, orbRadius * 1.15);
        grad.addColorStop(0, '#e0eaff');
        grad.addColorStop(0.22, '#b0c4de');
        grad.addColorStop(0.38, '#00fff7');
        grad.addColorStop(0.55, '#181c20');
        grad.addColorStop(0.7, '#ae00ff');
        grad.addColorStop(1, '#22223a');
        ctx.fillStyle = grad;
        ctx.shadowColor = '#00fff7';
        ctx.shadowBlur = 60;
        ctx.globalAlpha = 0.98;
        ctx.fill();
        // Neon rim
        ctx.lineWidth = 5;
        ctx.strokeStyle = 'rgba(0,255,247,0.6)';
        ctx.shadowColor = '#ae00ff';
        ctx.shadowBlur = 24;
        ctx.stroke();
        ctx.restore();
        t += 0.012;
        requestAnimationFrame(draw);
    }

    window.addEventListener('resize', () => {
        width = canvas.width = window.innerWidth;
        height = canvas.height = window.innerHeight;
        orbCenter.x = width/2;
        orbCenter.y = height/2;
    });

    draw();
}
