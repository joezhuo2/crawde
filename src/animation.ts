import idle from '../sprites/Crab_Idle.png';
import run from '../sprites/Crab_Run.png';
import ability from '../sprites/Crab_Ability.png';
import attackA from '../sprites/Crab_AttackA.png';
import attackB from '../sprites/Crab_AttackB.png';
import attackC from '../sprites/Crab_AttackC.png';
import attackAFx from '../sprites/Crab_AttackA_FX.png';
import attackBFx from '../sprites/Crab_AttackB_FX.png';
import attackCFx from '../sprites/Crab_AttackC_FX.png';

export const animations = {
  idle: { image: idle, frames: 4, columns: 4, fps: 5 },
  run: { image: run, frames: 6, columns: 4, fps: 11 },
  ability: { image: ability, frames: 10, columns: 4, fps: 12 },
  attackA: { image: attackA, frames: 10, columns: 4, fps: 13 },
  attackB: { image: attackB, frames: 6, columns: 4, fps: 11 },
  attackC: { image: attackC, frames: 11, columns: 4, fps: 12 },
  sleep: { image: idle, frames: 4, columns: 4, fps: 1 },
} as const;

export type Animation = keyof typeof animations;

const attackEffects = {
  attackA: { image: attackAFx, columns: 3, starts: [6] },
  attackB: { image: attackBFx, columns: 3, starts: [2] },
  attackC: { image: attackCFx, columns: 4, starts: [4, 7] },
} as const;

export function attackEffect(animation: Animation, frame: number) {
  if (!(animation in attackEffects)) return null;
  const effect = attackEffects[animation as keyof typeof attackEffects];
  const strike = effect.starts.findIndex(start => frame >= start && frame < start + 3);
  return strike < 0 ? null : { image: effect.image, columns: effect.columns, frame: strike * 3 + frame - effect.starts[strike] };
}

if (typeof Image !== 'undefined') {
  for (const src of new Set([...Object.values(animations).map(a => a.image), attackAFx, attackBFx, attackCFx])) new Image().src = src;
}

export function framePosition(frame: number, columns: number, scale: number) {
  // Keep the shared 32×24 character region, excluding unused headroom.
  return `${-(frame % columns * 64 + 16) * scale}px ${-(Math.floor(frame / columns) * 64 + 28) * scale}px`;
}

export function randomAttack(random = Math.random): Animation {
  return (['attackA', 'attackB', 'attackC'] as const)[Math.floor(random() * 3)];
}

export function clickAnimation(random = Math.random): Animation {
  return (['ability', 'attackA', 'attackB', 'attackC'] as const)[Math.floor(random() * 4)];
}
