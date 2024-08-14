import type { Position } from '../types/Vehicle'

export class Region {
  points: Array<Position>
  length: number

  constructor(points: Array<Position>) {
    this.points = points
    this.length = points.length
  }

  centroid() {
    let lat = 0,
      lng = 0

    lat = this.points.map((p) => p.lat).reduce((a, b) => a + b, 0)
    lng = this.points.map((p) => p.lng).reduce((a, b) => a + b, 0)
    return { lat: lat / this.points.length, lng: lng / this.points.length }
  }
}

export function distance(pos1: Position, pos2: Position) {
  return Math.sqrt(Math.pow(pos1.lat + pos2.lat, 2) + Math.pow(pos1.lng + pos2.lng, 2))
}
