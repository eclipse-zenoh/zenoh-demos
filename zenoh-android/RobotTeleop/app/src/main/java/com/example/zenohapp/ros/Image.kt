package com.example.zenohapp.ros

import android.graphics.Bitmap
import android.graphics.Color
import com.example.zenohapp.cdr.CDRInputStream
import java.nio.IntBuffer


class Image constructor(
    var header: Header,
    var height: UInt,
    var width: UInt,
    var encoding: String,
    var isBigendian: UByte,
    var step: UInt,
    var data: ByteArray
) {
    constructor(stream: CDRInputStream) : this(
        Header(stream),
        stream.readUInt(),
        stream.readUInt(),
        stream.readString(),
        stream.readUByte(),
        stream.readUInt(),
        stream.readByteArray(),
    )

    fun toBitmap() : Bitmap {
        val nrOfPixels = data.size / 3; // Three bytes per pixel.
        var pixels = IntArray(nrOfPixels){0}
        for(i in 0..< nrOfPixels) {
//            val b = data[3*i].toInt()
//            val g = data[3*i + 1].toInt()
//            val r = data[3*i + 2].toInt()

            val r = (0xFF and data.get(3 * i).toInt())
            val g = (0xFF and data.get(3 * i + 1).toInt())
            val b = (0xFF and data.get(3 * i + 2).toInt())

            pixels[i] = Color.rgb(r,g,b)
        }
        var bitmap = Bitmap.createBitmap(width.toInt(), height.toInt(), Bitmap.Config.ARGB_8888)
        bitmap.copyPixelsFromBuffer(IntBuffer.wrap(pixels))
        return bitmap
    }
}