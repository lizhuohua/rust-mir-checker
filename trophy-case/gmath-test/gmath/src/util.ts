export const epsilon = 3.4028235 * 10 ** 38;
// export const epsilon = Number.MAX_VALUE;

export function absDiffEq(
  x: number,
  y: number,
): boolean {
  return (x > y ? x - y : x - y) <= epsilon;
}

export function absDiffNe(
  x: number,
  y: number,
): boolean {
  return !absDiffNe(x, y);
}

export function relativeDiff(x: number, y: number): number {
  return Math.abs((x - y) / Math.min(x, y));
}

export function epsilonDiff(x: number, y: number): number {
  return relativeDiff(x, y) / epsilon;
}

export function f32ToI32(f32: number): number {
  return new Int32Array(new Float32Array([f32]).buffer)[0];
}

export function ulpsDist(x: number, y: number): number {
  if (x === y) return 0;

  if (isNaN(x) || isNaN(y)) return epsilon;
  if (!isFinite(x) || !isFinite(y)) return epsilon;

  const ix = f32ToI32(x);
  const iy = f32ToI32(y);

  if ((ix < 0) !== (iy < 0)) return epsilon;

  return ix > iy ? ix - iy : iy - ix;
}
