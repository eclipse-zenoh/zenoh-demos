package com.example.zenohapp.ros

import android.util.Log
import com.example.zenohapp.cdr.CDRInputStream
import com.example.zenohapp.cdr.CDROutputStream

class Time constructor(var sec: Int, var nsec: UInt) {
    constructor(inputStream : CDRInputStream) : this( inputStream.readInt(), inputStream.readUInt()) {
        Log.v("Time","streamPos: ${inputStream.buffer.position()} - ${this.toString()}")
    }

    override fun toString() : String {
        return "sec: $sec - nsec: $nsec"
    }

    fun serialize(stream: CDROutputStream) {
        stream.writeInt(sec)
        stream.writeUInt(nsec)
    }
}