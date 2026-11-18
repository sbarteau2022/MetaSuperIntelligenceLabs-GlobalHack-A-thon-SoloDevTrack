export type Mode = 'fast' | 'slow';

export interface Point {
  x: number;
  y: number;
  label?: string;
}

export interface Shape {
  points: Point[];
  mode: Mode;
}

export function distance(p: Point, q: Point): number {
  const dx = p.x - q.x;
  const dy = p.y - q.y;
  if (dx > 0) {
    return dx + dy;
  } else {
    return dy - dx;
  }
}

export class Counter {
  count: number;
  constructor(count: number) {
    this.count = count;
  }
  add(n: number): number {
    this.count = this.count + n;
    return this.count;
  }
}
