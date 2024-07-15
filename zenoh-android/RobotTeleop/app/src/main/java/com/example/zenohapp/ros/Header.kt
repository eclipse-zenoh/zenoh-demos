package com.example.zenohapp.ros

import android.util.Log
import com.example.zenohapp.cdr.CDRInputStream
import com.example.zenohapp.cdr.CDROutputStream
import com.example.zenohapp.ros.Time

class Header constructor(var time: Time, var frameId: String) {

    constructor(inputStream : CDRInputStream) : this( Time(inputStream), inputStream.readString()) {
        Log.v("Header","streamPos: ${inputStream.buffer.position()} - ${this.toString()}")
    }

    override fun toString() : String {
        return "Time: ${time.toString()} - frame_id: $frameId"
    }

    fun serialize(stream: CDROutputStream) {
        time.serialize(stream)
        stream.writeString(frameId)
    }
}