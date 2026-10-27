import idle from '../sprites/Crab_Idle.png';
import run from '../sprites/Crab_Run.png';
import ability from '../sprites/Crab_Ability.png';
import attackA from '../sprites/Crab_AttackA.png';
import attackB from '../sprites/Crab_AttackB.png';
import attackAFx from '../sprites/Crab_AttackA_FX.png';
import attackBFx from '../sprites/Crab_AttackB_FX.png';

export const animations = {
  idle: { image: idle, frames: 4, columns: 4, fps: 5 },
  run: { image: run, frames: 6, columns: 4, fps: 11 },
  ability: { image: ability, frames: 10, columns: 4, fps: 12 },
  attackA: { image: attackA, frames: 10, columns: 4, fps: 13 },
  attackB: { image: attackB, frames: 6, columns: 4, fps: 11 },
  sleep: { image: idle, frames: 4, columns: 4, fps: 1 },
} as const;

export type Animation = keyof typeof animations;

const attackEffects = {
  attackA: { image: attackAFx, start: 6 },
  attackB: { image: attackBFx, start: 2 },
} as const;

export function attackEffect(animation: Animation, frame: number) {
  if (animation !== 'attackA' && animation !== 'attackB') return null;
  const effect = attackEffects[animation];
  const effectFrame = frame - effect.start;
  return effectFrame >= 0 && effectFrame < 3 ? { image: effect.image, frame: effectFrame } : null;
}

export function framePosition(frame: number, columns: number, scale: number) {
  // Keep the shared 32×24 character region, excluding unused headroom.
  return `${-(frame % columns * 64 + 16) * scale}px ${-(Math.floor(frame / columns) * 64 + 28) * scale}px`;
}

export function clickAnimation(random = Math.random): Animation {
  return (['ability', 'attackA', 'attackB'] as const)[Math.floor(random() * 3)];
}
