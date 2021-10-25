import { Angle, Rad } from "./angle.ts";
import { Matrix4 } from "./matrix4.ts";
import { absDiffEq } from "./util.ts";

export class Perspective {
  left: number;
  right: number;
  bottom: number;
  top: number;
  near: number;
  far: number;

  constructor(
    left: number,
    right: number,
    bottom: number,
    top: number,
    near: number,
    far: number,
  ) {
    this.left = left;
    this.right = right;
    this.bottom = bottom;
    this.top = top;
    this.near = near;
    this.far = far;
  }

  toMatrix4(): Matrix4 {
    if (this.left > this.right) {
      throw new RangeError(
        `perspective.left (${this.right}) cannot be greater than perspective.right (${this.right})`,
      );
    }
    if (this.bottom > this.top) {
      throw new RangeError(
        `perspective.bottom (${this.bottom}) cannot be greater than perspective.top (${this.top})`,
      );
    }
    if (this.near > this.far) {
      throw new RangeError(
        `perspective.near (${this.near}) cannot be greater than perspective.far (${this.far})`,
      );
    }

    const c0r0 = (2 * this.near) / (this.right - this.left);
    const c0r1 = 0;
    const c0r2 = 0;
    const c0r3 = 0;

    const c1r0 = 0;
    const c1r1 = (2 * this.near) / (this.top - this.bottom);
    const c1r2 = 0;
    const c1r3 = 0;

    const c2r0 = (this.right + this.left) / (this.right - this.left);
    const c2r1 = (this.top + this.bottom) / (this.top - this.bottom);
    const c2r2 = -(this.far + this.near) / (this.far - this.near);
    const c2r3 = -1;

    const c3r0 = 0;
    const c3r1 = 0;
    const c3r2 = -(2 * this.far * this.near) / (this.far - this.near);
    const c3r3 = 0;

    return Matrix4.from(
      c0r0,
      c0r1,
      c0r2,
      c0r3,
      c1r0,
      c1r1,
      c1r2,
      c1r3,
      c2r0,
      c2r1,
      c2r2,
      c2r3,
      c3r0,
      c3r1,
      c3r2,
      c3r3,
    );
  }
}

export class PerspectiveFov {
  fovy: Rad;
  aspect: number;
  near: number;
  far: number;

  constructor(
    fovy: Angle,
    aspect: number,
    near: number,
    far: number,
  ) {
    fovy = fovy.toRad();

    this.fovy = fovy;
    this.aspect = aspect;
    this.near = near;
    this.far = far;
  }

  toPerspective(): Perspective {
    const angle = this.fovy.div(2);
    const ymax = this.near * angle.tan();
    const xmax = ymax * this.aspect;

    return new Perspective(-xmax, xmax, -ymax, ymax, this.near, this.far);
  }

  toMatrix4(): Matrix4 {
    if (this.fovy.value < 0) {
      throw new RangeError(
        `The vertical field of view cannot be below zero, found ${this.fovy.toString()}`,
      );
    }
    if (this.fovy.value > Rad.turn / 2) {
      throw new RangeError(
        `The vertical field of view cannot be greater than a half turn, found ${this.fovy.toString()}`,
      );
    }
    if (absDiffEq(Math.abs(this.aspect), 0)) {
      throw new RangeError(
        `The absolute aspect ratio cannot be zero, found ${
          Math.abs(this.aspect)
        }`,
      );
    }
    if (this.near < 0) {
      throw new RangeError(
        `The near plane distance cannot be below zero, found ${this.near}`,
      );
    }
    if (this.far < 0) {
      throw new RangeError(
        `The far plane distance cannot be below zero, found ${this.far}`,
      );
    }
    if (absDiffEq(this.far, this.near)) {
      throw new RangeError(
        `The far plane (${this.far}) and near plane (${this.near}) are too close`,
      );
    }

    const f = this.fovy.div(2).cot();

    const c0r0 = f / this.aspect;
    const c0r1 = 0;
    const c0r2 = 0;
    const c0r3 = 0;

    const c1r0 = 0;
    const c1r1 = f;
    const c1r2 = 0;
    const c1r3 = 0;

    const c2r0 = 0;
    const c2r1 = 0;
    const c2r2 = (this.far + this.near) / (this.far - this.near);
    const c2r3 = -1;

    const c3r0 = 0;
    const c3r1 = 0;
    const c3r2 = (2 * this.far * this.near) / (this.near - this.far);
    const c3r3 = 0;

    return Matrix4.from(
      c0r0,
      c0r1,
      c0r2,
      c0r3,
      c1r0,
      c1r1,
      c1r2,
      c1r3,
      c2r0,
      c2r1,
      c2r2,
      c2r3,
      c3r0,
      c3r1,
      c3r2,
      c3r3,
    );
  }
}

export class Orthographic {
  left: number;
  right: number;
  bottom: number;
  top: number;
  near: number;
  far: number;

  constructor(
    left: number,
    right: number,
    bottom: number,
    top: number,
    near: number,
    far: number,
  ) {
    this.left = left;
    this.right = right;
    this.bottom = bottom;
    this.top = top;
    this.near = near;
    this.far = far;
  }

  toMatrix4(): Matrix4 {
    const c0r0 = 2 / (this.right - this.left);
    const c0r1 = 0;
    const c0r2 = 0;
    const c0r3 = 0;

    const c1r0 = 0;
    const c1r1 = 2 / (this.top - this.bottom);
    const c1r2 = 0;
    const c1r3 = 0;

    const c2r0 = 0;
    const c2r1 = 0;
    const c2r2 = -2 / (this.far - this.near);
    const c2r3 = 0;

    const c3r0 = -(this.right + this.left) / (this.right - this.left);
    const c3r1 = -(this.top + this.bottom) / (this.top - this.bottom);
    const c3r2 = -(this.far + this.near) / (this.far - this.near);
    const c3r3 = 1;

    return Matrix4.from(
      c0r0,
      c0r1,
      c0r2,
      c0r3,
      c1r0,
      c1r1,
      c1r2,
      c1r3,
      c2r0,
      c2r1,
      c2r2,
      c2r3,
      c3r0,
      c3r1,
      c3r2,
      c3r3,
    );
  }
}
