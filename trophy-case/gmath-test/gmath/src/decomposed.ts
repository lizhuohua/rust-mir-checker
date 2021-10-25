import { Angle } from "./angle.ts";
import { Quaternion } from "./quaternion.ts";
import { Vector2 } from "./vector2.ts";
import { Vector3 } from "./vector3.ts";

export interface Decomposed2 {
  scale: number;
  rot: Angle;
  disp: Vector2;
}

export interface Decomposed3 {
  scale: number;
  rot: Quaternion;
  disp: Vector3;
}
