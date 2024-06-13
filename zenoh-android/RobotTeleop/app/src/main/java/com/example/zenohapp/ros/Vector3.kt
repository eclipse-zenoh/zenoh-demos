package com.example.zenohapp.ros

import com.example.zenohapp.cdr.CDROutputStream

class Vector3 constructor(var x: Double, var y: Double, var z: Double) {

    public fun serialize(stream: CDROutputStream) {
        stream.writeDouble(x)
        stream.writeDouble(y)
        stream.writeDouble(z)
    }

    public fun isZeros(): Boolean {
        return (x == 0.0 && y == 0.0 && z == 0.0)
    }


}