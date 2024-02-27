package com.example.zenohapp.cdr

import android.util.Log
import com.example.zenohapp.ui.examples.TeleopFragment
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.HexFormat

class CDROutputStream {
    var buffer: ByteBuffer

    constructor() {
        buffer = ByteBuffer.allocate(256)
        buffer.order(ByteOrder.LITTLE_ENDIAN)
        buffer.put(0)
        buffer.put(1)
        buffer.put(0)
        buffer.put(0)
    }

    fun writeDouble(value: Double) {
        align(4)
        buffer.putDouble(value)
    }

    fun writeZero(padding: Int) {
        for (i in 0..padding -1 ) {
            buffer.put(0)
        }
    }

    fun writeByte(b: Byte) {
        buffer.put(b)
    }

    fun writeUShort(value: UShort) {
        align(2)
        buffer.putShort(value.toShort())
    }

    fun writeInt(v: Int) {
        align(4)
        buffer.putInt(v)
    }

    fun writeBool(v: Boolean) {
        var bool : Byte = 0
        if (v) {
            bool = 1
        }
        buffer.put(bool)
    }

    fun writeUInt(v: UInt) {
        align(4)
        buffer.putInt(v.toInt())
    }

    fun writeString(v: String) {
        writeInt(v.length + 1)
        var arr = v.toCharArray()

        for (i in 0..(v.length-1)) {
            buffer.put(arr[i].code.toByte())
        }

        buffer.put(0)
    }


    private fun align(align: Int) {
        if (align > 1) {
            val padding = (-buffer.position() + 4) and (align - 1)
            if (padding > 0) {
                writeZero(padding)
            }
        }
    }

    fun dump() {
        val bytes = buffer.array()
        var str_repr = ""
        for (b in 0..buffer.position()-1) {
            str_repr += String.format(" %02X ", bytes[b])
        }
        Log.v(TAG,"Buffer is: $str_repr")
    }

    companion object {
        private val TAG = CDROutputStream::javaClass.name
    }
}
