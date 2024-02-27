package com.example.zenohapp.cdr

import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.charset.Charset

class CDRInputStream(var buffer: ByteBuffer) {


    val CDR_BE: UByte = 0u
    val CDR_LE: UByte = 1u

    init {
        readEncodingFlags()
    }
    private fun readEncodingFlags() {
        buffer.order(ByteOrder.LITTLE_ENDIAN)
        buffer.get()
        buffer.get()
        buffer.get()
        buffer.get()
    }

    private fun align(align: Int) {
        if (align > 1) {
            val padding = (-buffer.position() and (align - 1))
            if (padding != 0) {
                buffer.position(buffer.position() + padding)
            }
        }
    }

    fun readBoolean(): Boolean {
        return (buffer.get().toInt() != 0)
    }

    fun readFloat(): Float {
        align(4)
        return buffer.getFloat()
    }

    fun readInt(): Int {
        align(4)
        return buffer.getInt()
    }

    fun readUInt(): UInt {
        return readInt().toUInt()
    }

    fun readString(): String {
        val length = readInt()
        if (length < 1) {
            return ""
        }

        buffer.position((buffer.position() + length))
        return String(
            buffer.array(),
            (buffer.position() - length),
            (length - 1),
            Charset.defaultCharset()
        )
    }

    fun readUByte() : UByte {
        return buffer.get().toUByte()
    }


    fun readFloatArray() : Array<Float> {
        val length = readInt()
        var array = Array<Float>(length){0.0f}

        if (length == 0) {
            return array
        }
        for (i in 0..length) {
            array[i] = readFloat()
        }
        return array
    }

    fun readByteArray() : ByteArray {
        val length = readInt()
        return buffer.array().copyOfRange(buffer.position(), buffer.position()+length)
    }



}