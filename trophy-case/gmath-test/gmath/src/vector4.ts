import { Vector3 } from "./vector3.ts";

export interface Point4 {
  x: number;
  y: number;
  z: number;
  w: number;
}

export class Vector4 implements Point4 {
  #internal = new Float32Array(4);

  get [0](): number {
    return this.#internal[0];
  }

  set [0](val: number) {
    this.#internal[0] = val;
  }

  get [1](): number {
    return this.#internal[1];
  }

  set [1](val: number) {
    this.#internal[1] = val;
  }

  get [2](): number {
    return this.#internal[2];
  }

  set [2](val: number) {
    this.#internal[2] = val;
  }

  get [3](): number {
    return this.#internal[3];
  }

  set [3](val: number) {
    this.#internal[3] = val;
  }

  get x(): number {
    return this.#internal[0];
  }

  set x(val: number) {
    this.#internal[0] = val;
  }

  get y(): number {
    return this.#internal[1];
  }

  set y(val: number) {
    this.#internal[1] = val;
  }

  get z(): number {
    return this.#internal[2];
  }

  set z(val: number) {
    this.#internal[2] = val;
  }

  get w(): number {
    return this.#internal[3];
  }

  set w(val: number) {
    this.#internal[3] = val;
  }

  /** A Vector4 with all values set to Number.NEGATIVE_INFINITY */
  static negativeInfinity(): Vector4 {
    return new Vector4(Number.NEGATIVE_INFINITY);
  }

  /** A Vector4 with all values set to Number.POSITIVE_INFINITY */
  static positiveInfinity(): Vector4 {
    return new Vector4(Number.POSITIVE_INFINITY);
  }

  /** A Vector4 with all values set to 0 */
  static zero(): Vector4 {
    return new Vector4(0);
  }

  /** A Vector4 with all values set to 1 */
  static one(): Vector4 {
    return new Vector4(1);
  }

  constructor();
  constructor(x: number);
  constructor(x: number, y: number, z: number, w: number);
  constructor(x?: number, y?: number, z?: number, w?: number) {
    if (x !== undefined) {
      this.x = x;
      this.y = y ?? x;
      this.z = z ?? x;
      this.w = w ?? x;
    }
  }

  /** Creates a new Vector2 with the same values */
  clone(): Vector4 {
    return new Vector4(this.x, this.y, this.z, this.w);
  }

  /** The magnitude of this Vector4 */
  mag(): number {
    return Math.hypot(this.x, this.y, this.z, this.w);
  }

  /** The squared magnitude of this Vector4 */
  mag2(): number {
    return this.x ** 2 + this.y ** 2 + this.z ** 2 + this.w ** 2;
  }

  /** Returns a new Vector2 with the same direction, but with a magnitude of 1 */
  normal(): Vector4 {
    return this.div(this.mag());
  }

  /** Truncates this Vector4 to a Vector3 dropping the nth value */
  truncN(n: 0 | 1 | 2 | 3): Vector3 {
    switch (n) {
      case 0:
        return new Vector3(this.y, this.z, this.w);
      case 1:
        return new Vector3(this.x, this.z, this.w);
      case 2:
        return new Vector3(this.x, this.y, this.w);
      case 3:
        return new Vector3(this.x, this.y, this.z);
    }
  }

  /** Truncates this Vector4 to a Vector3 dropping the w value */
  trunc(): Vector3 {
    return new Vector3(this.x, this.y, this.z);
  }

  clamp(length: number): Vector4 {
    return this.normal().mul(length);
  }

  /** Calculates the dot product of this Vector4 */
  dot(other: Vector4): number {
    const { x, y, z, w } = this.mul(other);
    return x + y + z + w;
  }

  /** Linearly interpolates between this and the specified Vector4 */
  lerp(other: Vector4, alpha: number): Vector4 {
    return this.add(other.sub(this).mul(alpha));
  }

  /** Sets the x, y, z and w of this Vector4 to the specified Vector4 x, y, z and w values */
  set(other: Vector4): Vector4 {
    this.x = other.x;
    this.y = other.y;
    this.z = other.z;
    this.w = other.w;

    return this;
  }

  /** Adds this Vector4 to the specified Vector4 or scalar */
  add(other: number | Vector4): Vector4 {
    const { x, y, z, w } = typeof other === "number"
      ? { x: other, y: other, z: other, w: other }
      : other;

    return new Vector4(this.x + x, this.y + y, this.z + z, this.w + w);
  }

  /** Subtracts this Vector4 from the specified Vector4 or scalar */
  sub(other: number | Vector4): Vector4 {
    const { x, y, z, w } = typeof other === "number"
      ? { x: other, y: other, z: other, w: other }
      : other;

    return new Vector4(this.x - x, this.y - y, this.z - z, this.w - w);
  }

  /** Multiplies this Vector4 with the specified Vector4 or scalar */
  mul(other: number | Vector4): Vector4 {
    const { x, y, z, w } = typeof other === "number"
      ? { x: other, y: other, z: other, w: other }
      : other;

    return new Vector4(this.x * x, this.y * y, this.z * z, this.w * w);
  }

  /** Divides this Vector4 with the specified Vector4 or scalar */
  div(other: number | Vector4): Vector4 {
    const { x, y, z, w } = typeof other === "number"
      ? { x: other, y: other, z: other, w: other }
      : other;

    return new Vector4(this.x / x, this.y / y, this.z / z, this.w / w);
  }

  /** Negates the values of this Vector4 */
  neg(): Vector4 {
    return new Vector4(-this.x, -this.y, -this.z, -this.w);
  }

  /** Calculates the midpoint between two Vector4 */
  midpoint(other: Vector4): Vector4 {
    return other.sub(this).div(2).add(this);
  }

  /** Checks equality between two Vector4 */
  eq(other: Vector4): boolean {
    return this.x === other.x && this.y === other.y && this.z === other.z &&
      this.w === other.w;
  }

  /** Checks if the Vector4 is finite */
  isFinite(): boolean {
    return isFinite(this.x) && isFinite(this.y) && isFinite(this.z) &&
      isFinite(this.w);
  }

  /** Converts the Vector4 to a string */
  toString(): string {
    return `Vector4 { x: ${this[0]}, y: ${this[1]}, z: ${this[2]}, w: ${
      this[3]
    } }`;
  }

  /** Converts the Vector4 to a tuple of numbers */
  toArray(): [number, number, number, number] {
    return [this[0], this[1], this[2], this[3]];
  }

  /** Converts the Vector to a Float32Array */
  toFloat32Array(): Float32Array {
    return this.#internal;
  }
}
