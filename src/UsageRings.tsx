import type { Usage, UsageWindow } from './types';
import { hasUsage, ringDash, usageLevel, usageText } from './usageRing';

const SIZE = 22;
const STROKE = 3;
const RADIUS = (SIZE - STROKE) / 2;
const CENTER = SIZE / 2;

function Ring({ label, name, window, now }: { label: string; name: string; window: UsageWindow; now: number }) {
  const text = usageText(name, window, now);
  return <span className={`usage-ring ${usageLevel(window.pct)}`} tabIndex={0} role="img" aria-label={text}>
    <span className="usage-dial" aria-hidden="true">
      <svg width={SIZE} height={SIZE} viewBox={`0 0 ${SIZE} ${SIZE}`}>
        <circle className="usage-track" cx={CENTER} cy={CENTER} r={RADIUS} strokeWidth={STROKE} />
        <circle className="usage-arc" cx={CENTER} cy={CENTER} r={RADIUS} strokeWidth={STROKE} strokeDasharray={ringDash(window.pct, RADIUS)} transform={`rotate(-90 ${CENTER} ${CENTER})`} />
      </svg>
      <span className="usage-label">{label}</span>
    </span>
    <span className="usage-tip" aria-hidden="true">{text}</span>
  </span>;
}

export function UsageRings({ usage }: { usage?: Usage | null }) {
  if (!hasUsage(usage)) return null;
  const now = Date.now();
  return <div className="usage-rings" role="group" aria-label="Plan usage">
    {usage.fiveHour && <Ring label="5h" name="5h" window={usage.fiveHour} now={now} />}
    {usage.sevenDay && <Ring label="7d" name="7d" window={usage.sevenDay} now={now} />}
  </div>;
}
