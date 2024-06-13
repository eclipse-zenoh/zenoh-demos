package com.example.zenohapp.ros

import com.example.zenohapp.cdr.CDROutputStream

class Twist(var linear: Vector3, var angular: Vector3) {

    public fun serialize(stream: CDROutputStream) {
        linear.serialize(stream)
        angular.serialize(stream)
    }

    public fun isStopped(): Boolean {
        return (linear.isZeros() && angular.isZeros())
    }
}