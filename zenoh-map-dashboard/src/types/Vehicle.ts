export interface Vehicle {
  position: Position
  speed: number
  color: string
  id: string
  kind: string
  timestamp: string
}

export interface Position {
  lat: number
  lng: number
}
