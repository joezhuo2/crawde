import idle from '../sprites/Crab_Idle.png';
import run from '../sprites/Crab_Run.png';
import ability from '../sprites/Crab_Ability.png';
import attackA from '../sprites/Crab_AttackA.png';
import attackB from '../sprites/Crab_AttackB.png';

export const animations = {
  idle: { image: idle, frames: 4, columns: 4, fps: 8 },
  run: { image: run, frames: 6, columns: 4, fps: 11 },
  ability: { image: ability, frames: 10, columns: 4, fps: 12 },
  attackA: { image: attackA, frames: 10, columns: 4, fps: 13 },
  attackB: { image: attackB, frames: 6, columns: 4, fps: 11 },
  sleep: { image: idle, frames: 4, columns: 4, fps: .8 },
} as const;

export type Animation = keyof typeof animations;

export function framePosition(frame: number, columns: number, scale: number) {
  // All source cells are 64px; crop to the shared 32px character region.
  return `${-(frame % columns * 64 + 16) * scale}px ${-(Math.floor(frame / columns) * 64 + 20) * scale}px`;
}
