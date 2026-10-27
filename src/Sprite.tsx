import { useEffect, useState } from 'react';
import { animations, attackEffect, framePosition, type Animation } from './animation';

interface Props {
  animation: Animation;
  scale?: number;
  reducedMotion?: boolean;
  facing?: number;
  onComplete?: () => void;
}

export function Sprite({ animation, scale = 3, reducedMotion = false, facing = 1, onComplete }: Props) {
  const [frame, setFrame] = useState(0);
  const [systemReduced, setSystemReduced] = useState(() => matchMedia('(prefers-reduced-motion: reduce)').matches);
  const spec = animations[animation];
  const effect = reducedMotion || systemReduced ? null : attackEffect(animation, frame);

  useEffect(() => {
    const media = matchMedia('(prefers-reduced-motion: reduce)');
    const change = () => setSystemReduced(media.matches);
    media.addEventListener('change', change);
    return () => media.removeEventListener('change', change);
  }, []);

  useEffect(() => {
    setFrame(0);
    const oneShot = ['ability', 'attackA', 'attackB'].includes(animation);
    let tick = 0;
    const timer = window.setInterval(() => {
      tick++;
      if (oneShot && tick >= spec.frames) {
        window.clearInterval(timer);
        onComplete?.();
      } else if (!reducedMotion && !systemReduced) setFrame(tick % spec.frames);
    }, 1000 / spec.fps);
    return () => window.clearInterval(timer);
  }, [animation, spec, reducedMotion, systemReduced, onComplete]);

  return <span className="sprite-wrap" style={{ width: 32 * scale, height: 24 * scale }}>
    <span className="sprite" role="img" aria-label={`Crawde ${animation}`} style={{
      width: 32 * scale, height: 24 * scale,
      backgroundImage: `url(${spec.image})`,
      backgroundSize: `${spec.columns * 64 * scale}px auto`,
      backgroundPosition: framePosition(frame, spec.columns, scale),
      transform: `scaleX(${facing})`,
    }}>
      {effect && <span className="sprite-fx" aria-hidden="true" style={{
        // The character viewport starts at (16, 28) inside the original cell.
        // Render the whole FX cell so sparks beyond that viewport stay visible.
        left: -16 * scale, top: -28 * scale,
        width: 64 * scale, height: 64 * scale,
        backgroundImage: `url(${effect.image})`,
        backgroundSize: `${3 * 64 * scale}px auto`,
        backgroundPosition: `${-effect.frame * 64 * scale}px 0px`,
      }} />}
    </span>
    {animation === 'sleep' && <span className={`sleep-trail ${reducedMotion || systemReduced ? 'still' : ''}`} aria-hidden="true" style={{ left: facing > 0 ? '72%' : '24%', '--sleep-direction': facing } as React.CSSProperties}>
      {[0, 1, 2].map(index => <span key={index} style={{ animationDelay: `${index * 3.2 / 3}s` }}>z</span>)}
    </span>}
  </span>;
}
