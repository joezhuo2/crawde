import { useEffect, useState } from 'react';
import { animations, framePosition, type Animation } from './animation';

interface Props {
  animation: Animation;
  scale?: number;
  reducedMotion?: boolean;
  facing?: number;
  onComplete?: () => void;
}

export function Sprite({ animation, scale = 3, reducedMotion = false, facing = 1, onComplete }: Props) {
  const [frame, setFrame] = useState(0);
  const [sleepStyle] = useState(() => Math.random() > .5 ? 'letters' : 'bubble');
  const spec = animations[animation];

  useEffect(() => {
    setFrame(0);
    const oneShot = ['ability', 'attackA', 'attackB'].includes(animation);
    let tick = 0;
    const timer = window.setInterval(() => {
      tick++;
      if (oneShot && tick >= spec.frames) {
        window.clearInterval(timer);
        onComplete?.();
      } else if (!reducedMotion) setFrame(tick % spec.frames);
    }, 1000 / spec.fps);
    return () => window.clearInterval(timer);
  }, [animation, spec, reducedMotion, onComplete]);

  return <span className="sprite-wrap" style={{ width: 32 * scale, height: 32 * scale }}>
    <span className="sprite" role="img" aria-label={`Crawde ${animation}`} style={{
      width: 32 * scale, height: 32 * scale,
      backgroundImage: `url(${spec.image})`,
      backgroundSize: `${spec.columns * 64 * scale}px auto`,
      backgroundPosition: framePosition(frame, spec.columns, scale),
      transform: `scaleX(${facing})`,
    }} />
    {animation === 'sleep' && <span className={`sleep ${sleepStyle}`} aria-hidden="true">{sleepStyle === 'bubble' ? '· z' : 'z Z z'}</span>}
  </span>;
}
