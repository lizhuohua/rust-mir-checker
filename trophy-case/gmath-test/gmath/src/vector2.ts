import { Rad } from "./angle.ts";
import { Vector3 } from "./vector3.ts";
import { Vector4 } from "./vector4.ts";

export interface Point2 {
  x: number;
  y: number;
}

export class Vector2 implements Point2 {
  #internal = new Float32Array(2);

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

  /** A Vector2 with all values set to Number.NEGATIVE_INFINITY */
  static negativeInfinity(): Vector2 {
    return new Vector2(Number.NEGATIVE_INFINITY);
  }

  /** A Vector2 with all values set to Number.POSITIVE_INFINITY */
  static positiveInfinity(): Vector2 {
    return new Vector2(Number.POSITIVE_INFINITY);
  }

  /** A Vector2 with all values set to 0 */
  static zero(): Vector2 {
    return new Vector2(0);
  }

  /** A Vector2 with all values set to 1 */
  static one(): Vector2 {
    return new Vector2(1);
  }

  /** Shorthand for writing Vector2(0, 1) */
  static up(): Vector2 {
    return new Vector2(0, 1);
  }

  /** Shorthand for writing Vector2(0, -1) */
  static down(): Vector2 {
    return new Vector2(0, -1);
  }

  /** Shorthand for writing Vector2(-1, 0) */
  static left(): Vector2 {
    return new Vector2(-1, 0);
  }

  /** Shorthand for writing Vector2(1, 0) */
  static right(): Vector2 {
    return new Vector2(1, 0);
  }

  constructor();
  constructor(x: number);
  constructor(x: number, y: number);
  constructor(x?: number, y?: number) {
    if (x !== undefined) {
      this.x = x;

      if (y !== undefined) {
        this.y = y;
      } else {
        this.y = x;
      }
    }
  }

  /** Creates a new Vector2 with the same values */
  clone(): Vector2 {
    return new Vector2(this.x, this.y);
  }

  /** The magnitude of this Vector2 */
  mag(): number {
    return Math.hypot(this.x, this.y);
  }

  /** The squared magnitude of this Vector2 */
  mag2(): number {
    return this.x ** 2 + this.y ** 2;
  }

  /** Returns a new Vector2 with the same direction, but with a magnitude of 1 */
  normal(): Vector2 {
    return this.div(this.mag());
  }

  /** Returns the angle of this Vector2 */
  angle(): Rad {
    return new Rad(Math.atan2(this.y, this.x));
  }

  /** Returns a new Vector2 with the same direction, but clamped to the specified length */
  clamp(length: number): Vector2 {
    return this.normal().mul(length);
  }

  /** Calculates the dot product of this Vector2 */
  dot(other: Vector2): number {
    const { x, y } = this.mul(other);
    return x + y;
  }

  /** Linearly interpolates between this and the specified Vector2 */
  lerp(other: Vector2, alpha: number): Vector2 {
    return this.add(other.sub(this).mul(alpha));
  }

  /** Sets the x and y of this Vector2 to the specified Vector2 x and y values */
  set(other: Vector2): Vector2 {
    this.x = other.x;
    this.y = other.y;

    return this;
  }

  /** Adds this Vector2 to the specified Vector2 or scalar */
  add(other: number | Vector2): Vector2 {
    const { x, y } = typeof other === "number" ? { x: other, y: other } : other;

    return new Vector2(this.x + x, this.y + y);
  }

  /** Subtracts this Vector2 from the specified Vector2 or scalar */
  sub(other: number | Vector2): Vector2 {
    const { x, y } = typeof other === "number" ? { x: other, y: other } : other;

    return new Vector2(this.x - x, this.y - y);
  }

  /** Multiplies this Vector2 with the specified Vector2 or scalar */
  mul(other: number | Vector2): Vector2 {
    const { x, y } = typeof other === "number" ? { x: other, y: other } : other;

    return new Vector2(this.x * x, this.y * y);
  }

  /** Divides this Vector2 with the specified Vector2 or scalar */
  div(other: number | Vector2): Vector2 {
    const { x, y } = typeof other === "number" ? { x: other, y: other } : other;

    return new Vector2(this.x / x, this.y / y);
  }

  /** Negates the values of this Vector2 */
  neg(): Vector2 {
    return new Vector2(-this.x, -this.y);
  }

  /** Calculates the midpoint between two Vector2 */
  midpoint(other: Vector2): Vector2 {
    return other.sub(this).div(2).add(this);
  }

  /** Checks equality between two Vector2 */
  eq(other: Vector2): boolean {
    return this.x === other.x && this.y === other.y;
  }

  /** Checks if the Vector2 is finite */
  isFinite(): boolean {
    return isFinite(this.x) && isFinite(this.y);
  }

  /** Create a Vector3 using this x and y and the provided z */
  extend3(z: number): Vector3 {
    return new Vector3(this.x, this.y, z);
  }

  /** Create a Vector4 using this x and y and the provided z and w */
  extend4(z: number, w: number): Vector4 {
    return new Vector4(this.x, this.y, z, w);
  }

  /** Converts the Vector2 to a string */
  toString(): string {
    return `Vector2 { x: ${this[0]}, y: ${this[1]} }`;
  }

  /** Converts the Vector2 to a tuple of numbers */
  toArray(): [number, number] {
    return [this[0], this[1]];
  }

  /** Converts the Vector to a Float32Array */
  toFloat32Array(): Float32Array {
    return this.#internal;
  }
}
